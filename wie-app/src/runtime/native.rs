use std::{
    collections::{HashMap, hash_map::Entry},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, RecvTimeoutError},
    },
    time::{Duration, Instant},
};

use anyhow::{Result, anyhow};
use futures::channel::oneshot;
use tauri::{AppHandle, Manager, State};

use wie::load_emulator;
use wie_backend::{
    AudioSink, DatabaseRepository as BackendDatabaseRepository, Event, Filesystem, Font, Instant as GuestInstant, KeyCode, Options, Platform, Screen,
};

use crate::{
    audio::{Audio, AudioSink as NativeAudioSink},
    database::DatabaseRepository,
    filesystem::SqliteFilesystem,
    screen::{NativeScreen, NativeView},
};

use super::{AppState, Clock, SessionEvent, SessionWorker};

pub(super) type StartedApp = u64;

pub(super) enum Command {
    Key(KeyCode, bool),
    ReleaseKeys,
    Volumes(f32, f32),
    #[cfg(mobile)]
    Suspend(bool),
    Stop,
}

impl AppState {
    pub fn release_keys(&self) {
        if let Some(session) = &self.runtime.lock().unwrap().session
            && let Some(commands) = &session.commands
        {
            let _ = commands.send(Command::ReleaseKeys);
        }
    }
}

#[tauri::command]
pub(crate) async fn key_event(state: State<'_, AppState>, session_id: u64, key: String, pressed: bool) -> Result<(), String> {
    let key = match key.as_str() {
        "UP" => KeyCode::UP,
        "DOWN" => KeyCode::DOWN,
        "LEFT" => KeyCode::LEFT,
        "RIGHT" => KeyCode::RIGHT,
        "OK" => KeyCode::OK,
        "0" => KeyCode::NUM0,
        "1" => KeyCode::NUM1,
        "2" => KeyCode::NUM2,
        "3" => KeyCode::NUM3,
        "4" => KeyCode::NUM4,
        "5" => KeyCode::NUM5,
        "6" => KeyCode::NUM6,
        "7" => KeyCode::NUM7,
        "8" => KeyCode::NUM8,
        "9" => KeyCode::NUM9,
        "#" => KeyCode::HASH,
        "*" => KeyCode::STAR,
        "CLR" => KeyCode::CLEAR,
        _ => return Err(format!("Unknown key: {key}")),
    };
    let runtime = state.runtime.lock().unwrap();
    if let Some(session) = &runtime.session
        && session.id == session_id
        && let Some(commands) = &session.commands
    {
        let _ = commands.send(Command::Key(key, pressed));
    }
    Ok(())
}

#[tauri::command]
pub(crate) async fn release_keys(state: State<'_, AppState>, session_id: u64) -> Result<(), String> {
    let runtime = state.runtime.lock().unwrap();
    if let Some(session) = &runtime.session
        && session.id == session_id
        && let Some(commands) = &session.commands
    {
        let _ = commands.send(Command::ReleaseKeys);
    }
    Ok(())
}

struct NativePlatform {
    app: AppHandle,
    screen: NativeScreen,
    audio: NativeAudioSink,
    database: DatabaseRepository,
    filesystem: SqliteFilesystem,
    clock: Arc<Mutex<Clock>>,
    exited: Arc<AtomicBool>,
    font: Font,
}

impl Platform for NativePlatform {
    fn font(&self) -> &Font {
        &self.font
    }

    fn screen(&self) -> &dyn Screen {
        &self.screen
    }

    fn now(&self) -> GuestInstant {
        self.clock.lock().unwrap().now()
    }

    fn database_repository(&self) -> &dyn BackendDatabaseRepository {
        &self.database
    }

    fn filesystem(&self) -> &dyn Filesystem {
        &self.filesystem
    }

    fn audio_sink(&self) -> Box<dyn AudioSink> {
        Box::new(self.audio.clone())
    }

    fn write_stdout(&self, bytes: &[u8]) {
        log::info!("{}", String::from_utf8_lossy(bytes));
    }

    fn write_stderr(&self, bytes: &[u8]) {
        log::warn!("{}", String::from_utf8_lossy(bytes));
    }

    fn exit(&self) {
        self.exited.store(true, Ordering::Release);
    }

    fn vibrate(&self, duration_ms: u64, intensity: u8) {
        #[cfg(mobile)]
        {
            use tauri_plugin_haptics::HapticsExt;
            if duration_ms != 0
                && intensity != 0
                && let Err(error) = self.app.haptics().vibrate(duration_ms.min(u64::from(u32::MAX)) as u32)
            {
                log::warn!("Vibration failed: {error}");
            }
        }
        #[cfg(desktop)]
        let _ = (&self.app, duration_ms, intensity);
    }
}

impl SessionWorker {
    pub(super) fn prepare(self, session_id: u64, filename: String, bytes: Vec<u8>) -> (StartedApp, NativeSessionWorker) {
        let view = self.app.state::<AppState>().view.clone();
        (
            session_id,
            NativeSessionWorker {
                session: self,
                view,
                filename,
                bytes,
            },
        )
    }
}

pub(super) struct NativeSessionWorker {
    session: SessionWorker,
    view: NativeView,
    filename: String,
    bytes: Vec<u8>,
}

impl NativeSessionWorker {
    pub(super) fn run(self, commands: Receiver<Command>, started: oneshot::Sender<()>) -> Result<()> {
        let Self {
            session:
                SessionWorker {
                    app,
                    store,
                    settings,
                    events,
                    initialized,
                    #[cfg(mobile)]
                    suspended,
                },
            view,
            filename,
            bytes,
        } = self;
        let warnings = events.clone();
        let mut audio = Audio::new(&app, settings.midi_volume, settings.pcm_volume, move |message| {
            let _ = warnings.send(SessionEvent::Warning { message });
        })?;
        let clock = Arc::new(Mutex::new(Clock::new()?));
        #[cfg(mobile)]
        {
            clock.lock().unwrap().set_paused(suspended);
            audio.pause(suspended)?;
        }
        let redraw = Arc::new(AtomicBool::new(true));
        let exited = Arc::new(AtomicBool::new(false));
        let platform = NativePlatform {
            app,
            screen: NativeScreen::new(view.clone(), 240, 320, redraw.clone()),
            audio: audio.sink(),
            database: DatabaseRepository { store: store.clone() },
            filesystem: SqliteFilesystem { store },
            clock: clock.clone(),
            exited: exited.clone(),
            font: Font::try_from_static(include_bytes!("../../../assets/neodgm.ttf"))?,
        };
        let result = (|| -> Result<()> {
            view.set_playing(true)?;
            let mut emulator = load_emulator(
                &filename,
                bytes,
                Box::new(platform),
                Options {
                    enable_gdbserver: false,
                    aot: None,
                    profile: None,
                },
            )?;
            initialized.store(true, Ordering::Release);
            let _ = started.send(());
            let mut keys = HashMap::new();
            let mut ready = false;
            #[cfg(mobile)]
            let mut paused = suspended;
            #[cfg(desktop)]
            let paused = false;
            loop {
                let command = match commands.recv_timeout(Duration::from_millis(if paused { 100 } else { 1 })) {
                    Ok(command) => Some(command),
                    Err(RecvTimeoutError::Timeout) => None,
                    Err(RecvTimeoutError::Disconnected) => break,
                };
                if let Some(command) = command {
                    match command {
                        Command::Key(key, pressed) if ready && !paused => {
                            if pressed {
                                if let Entry::Vacant(entry) = keys.entry(key) {
                                    entry.insert(Instant::now());
                                    emulator.handle_event(Event::Keydown(key));
                                }
                            } else if keys.remove(&key).is_some() {
                                emulator.handle_event(Event::Keyup(key));
                            }
                        }
                        Command::Key(..) => {}
                        Command::ReleaseKeys => {
                            for (key, _) in keys.drain() {
                                emulator.handle_event(Event::Keyup(key));
                            }
                        }
                        Command::Volumes(midi, pcm) => audio.set_volumes(midi, pcm)?,
                        #[cfg(mobile)]
                        Command::Suspend(suspended) => {
                            for (key, _) in keys.drain() {
                                emulator.handle_event(Event::Keyup(key));
                            }
                            clock.lock().unwrap().set_paused(suspended);
                            audio.pause(suspended)?;
                            paused = suspended;
                        }
                        Command::Stop => break,
                    }
                }
                if exited.load(Ordering::Acquire) {
                    break;
                }
                if let Some(error) = view.take_error() {
                    return Err(anyhow!(error));
                }
                if paused {
                    continue;
                }
                if redraw.swap(false, Ordering::AcqRel) {
                    emulator.handle_event(Event::Redraw);
                }
                for (&key, last) in &mut keys {
                    if last.elapsed() >= Duration::from_millis(100) {
                        emulator.handle_event(Event::Keyrepeat(key));
                        *last = Instant::now();
                    }
                }
                emulator.tick()?;
                if !ready && !emulator.is_preparing() {
                    ready = true;
                    events.send(SessionEvent::Ready)?;
                }
            }
            for (key, _) in keys {
                emulator.handle_event(Event::Keyup(key));
            }
            Ok(())
        })();
        audio.shutdown();
        result
    }
}
