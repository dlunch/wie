use std::{
    sync::mpsc::{self, Receiver, RecvTimeoutError, Sender},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use anyhow::{Context, Result, anyhow};
use tauri::AppHandle;
use wie_backend::AudioCommand;

#[cfg(target_os = "android")]
mod android;
#[cfg(target_vendor = "apple")]
mod apple;
#[cfg(all(target_os = "linux", not(test)))]
mod linux;
#[cfg(any(target_os = "linux", target_os = "windows"))]
mod midi;
#[cfg(not(target_vendor = "apple"))]
mod output;
mod schedule;
#[cfg(any(target_os = "android", test))]
mod smf;
#[cfg(all(target_os = "windows", not(test)))]
mod windows;

#[cfg(target_vendor = "apple")]
use apple::Output;
#[cfg(not(target_vendor = "apple"))]
use output::Output;
use schedule::{Clock, Due, Schedule};

type Warning = Box<dyn Fn(String) + Send>;
type Reply = Sender<Result<()>>;

enum Command {
    Audio(AudioCommand),
    Volumes(f32, f32, Reply),
    #[cfg(mobile)]
    Pause(Reply),
    #[cfg(mobile)]
    Resume(Reply),
    Shutdown,
}

#[derive(Clone)]
pub struct AudioSink(Sender<Command>);

impl wie_backend::AudioSink for AudioSink {
    fn send(&self, command: AudioCommand) {
        // A guest can retain its sink after the session owner has shut down.
        let _ = self.0.send(Command::Audio(command));
    }
}

pub struct Audio {
    tx: Sender<Command>,
    worker: Option<JoinHandle<()>>,
}

impl Audio {
    pub fn new(app: &AppHandle, midi_volume: f32, pcm_volume: f32, warning: impl Fn(String) + Send + 'static) -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        let (ready_tx, ready_rx) = mpsc::channel();
        let app = app.clone();
        let worker = thread::Builder::new().name("wie-audio".into()).spawn(move || {
            let warning: Warning = Box::new(warning);
            match Output::new(&app, midi_volume, pcm_volume, &warning) {
                Ok(output) => {
                    let _ = ready_tx.send(Ok(()));
                    run(rx, output, warning);
                }
                Err(error) => {
                    let _ = ready_tx.send(Err(error));
                }
            }
        })?;
        let mut audio = Self { tx, worker: Some(worker) };
        if let Err(error) = ready_rx.recv().context("Audio worker exited during startup").and_then(|r| r) {
            audio.shutdown();
            return Err(error);
        }
        Ok(audio)
    }

    pub fn sink(&self) -> AudioSink {
        AudioSink(self.tx.clone())
    }

    pub fn set_volumes(&self, midi: f32, pcm: f32) -> Result<()> {
        self.request(|reply| Command::Volumes(midi, pcm, reply))
    }

    #[cfg(mobile)]
    pub fn pause(&self, paused: bool) -> Result<()> {
        self.request(if paused { Command::Pause } else { Command::Resume })
    }

    fn request(&self, command: impl FnOnce(Reply) -> Command) -> Result<()> {
        let (tx, rx) = mpsc::channel();
        self.tx.send(command(tx)).map_err(|_| anyhow!("Audio worker has stopped"))?;
        rx.recv().context("Audio worker exited before acknowledging control")?
    }

    /// Called by the session worker, never the UI thread.
    pub fn shutdown(&mut self) {
        if let Some(worker) = self.worker.take() {
            let _ = self.tx.send(Command::Shutdown);
            if worker.join().is_err() {
                log::error!("Audio worker panicked");
            }
        }
    }
}

impl Drop for Audio {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn run(rx: Receiver<Command>, mut output: Output, warning: Warning) {
    let mut schedule = Schedule::default();
    let clock = Clock {
        origin: Instant::now(),
        paused_at: None,
    };
    #[cfg(mobile)]
    let mut clock = clock;
    loop {
        let timeout = if clock.paused_at.is_some() {
            None
        } else {
            schedule
                .deadline()
                .map(|deadline| deadline.saturating_sub(clock.elapsed(Instant::now())))
                .into_iter()
                .chain(output.has_tails().then_some(Duration::from_millis(20)))
                .min()
        };
        let command = match timeout {
            Some(timeout) => match rx.recv_timeout(timeout) {
                Ok(command) => Some(command),
                Err(RecvTimeoutError::Timeout) => None,
                Err(RecvTimeoutError::Disconnected) => break,
            },
            None => match rx.recv() {
                Ok(command) => Some(command),
                Err(_) => break,
            },
        };
        match command {
            Some(Command::Shutdown) => break,
            Some(Command::Audio(AudioCommand::Play { handle, sequence, repeat })) => {
                schedule.stop(handle);
                if let Err(error) = output.stop(handle) {
                    warning(format!("Audio stop failed: {error:#}"));
                }
                if let Err(error) = output.start(handle, &sequence, repeat) {
                    warning(format!("MIDI is unavailable for this playback; PCM audio will continue: {error:#}"));
                }
                schedule.play(handle, sequence, repeat, clock.elapsed(Instant::now()));
            }
            Some(Command::Audio(AudioCommand::Stop { handle })) => {
                schedule.stop(handle);
                if let Err(error) = output.stop(handle) {
                    warning(format!("Audio stop failed: {error:#}"));
                }
            }
            Some(Command::Volumes(midi, pcm, reply)) => {
                let _ = reply.send(output.set_volumes(midi, pcm));
            }
            #[cfg(mobile)]
            Some(Command::Pause(reply)) => {
                let result = if clock.paused_at.is_none() {
                    output.pause().map(|()| {
                        clock.paused_at = Some(Instant::now());
                    })
                } else {
                    Ok(())
                };
                let _ = reply.send(result);
            }
            #[cfg(mobile)]
            Some(Command::Resume(reply)) => {
                let result = if clock.paused_at.is_some() {
                    output.resume().map(|()| {
                        clock.resume(Instant::now());
                    })
                } else {
                    Ok(())
                };
                let _ = reply.send(result);
            }
            None => {}
        }
        if clock.paused_at.is_none() {
            for due in schedule.due(clock.elapsed(Instant::now())) {
                let result = match due {
                    Due::Event(handle, sequence, index) => output.event(handle, &sequence.events[index].data),
                    Due::End(handle) => output.finish(handle),
                };
                if let Err(error) = result {
                    warning(format!("Audio output failed: {error:#}"));
                }
            }
            output.reap();
        }
    }
}
