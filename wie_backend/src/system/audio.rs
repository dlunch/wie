use alloc::{boxed::Box, collections::BTreeMap, sync::Arc, vec::Vec};
use core::sync::atomic::{AtomicBool, Ordering};

use smaf_player::{SmafEvent, parse_smaf};

use crate::{System, audio_sink::AudioSink};

pub type AudioHandle = u32;
#[derive(Debug)]
pub enum AudioError {
    InvalidHandle,
    InvalidAudio,
}

enum AudioFile {
    Smaf(Vec<u8>),
}

pub struct Audio {
    sink: Arc<Box<dyn AudioSink>>,
    files: BTreeMap<AudioHandle, AudioFile>,
    playing: BTreeMap<AudioHandle, Arc<AtomicBool>>,
    last_audio_handle: AudioHandle,
}

impl Audio {
    pub fn new(sink: Box<dyn AudioSink>) -> Self {
        Self {
            sink: Arc::new(sink),
            files: BTreeMap::new(),
            playing: BTreeMap::new(),
            last_audio_handle: 0,
        }
    }

    pub fn load_smaf(&mut self, data: &[u8]) -> Result<AudioHandle, AudioError> {
        let audio_handle = self.last_audio_handle;

        self.last_audio_handle += 1;
        self.files.insert(audio_handle, AudioFile::Smaf(data.to_vec()));

        Ok(audio_handle)
    }

    pub fn play(&mut self, system: &System, audio_handle: AudioHandle) -> Result<(), AudioError> {
        match self.files.get(&audio_handle) {
            Some(AudioFile::Smaf(data)) => {
                let player = SmafPlayer::new(data);
                let mut system_clone = system.clone();
                let sink_clone = self.sink.clone();

                let stop_flag = Arc::new(AtomicBool::new(false));
                let stop_flag_clone = stop_flag.clone();
                self.playing.insert(audio_handle, stop_flag);

                // TODO use dedicated audio player task
                system.spawn(async move || {
                    player.play(&mut system_clone, &**sink_clone, &stop_flag_clone).await;

                    Ok(())
                });
            }
            None => return Err(AudioError::InvalidHandle),
        }

        Ok(())
    }

    pub fn stop(&mut self, audio_handle: AudioHandle) -> Result<(), AudioError> {
        match self.playing.remove(&audio_handle) {
            Some(stop_flag) => {
                stop_flag.store(true, Ordering::Relaxed);
            }
            None => return Err(AudioError::InvalidHandle),
        }

        Ok(())
    }

    pub fn close(&mut self, audio_handle: AudioHandle) -> Result<(), AudioError> {
        if self.files.remove(&audio_handle).is_none() {
            return Err(AudioError::InvalidHandle);
        }

        Ok(())
    }
}

pub struct SmafPlayer {
    events: Vec<(usize, SmafEvent)>,
}

impl SmafPlayer {
    pub fn new(data: &[u8]) -> Self {
        Self { events: parse_smaf(data) }
    }

    pub async fn play(&self, system: &mut System, sink: &dyn AudioSink, stop_flag: &AtomicBool) {
        let mut active_notes: Vec<(u8, u8)> = Vec::new();

        let start_time = system.platform().now();
        for (time, event) in &self.events {
            if stop_flag.load(Ordering::Relaxed) {
                for (channel, note) in &active_notes {
                    sink.midi_note_off(*channel, *note, 0);
                }
                break;
            }

            let now = system.platform().now();
            if (*time as u64) > now - start_time {
                system.sleep(((*time as u64) - (now - start_time)) as _).await;
            }

            match event {
                SmafEvent::Wave {
                    channel,
                    sampling_rate,
                    data,
                } => {
                    sink.play_wave(*channel, *sampling_rate, data);
                }
                SmafEvent::MidiNoteOn { channel, note, velocity } => {
                    sink.midi_note_on(*channel, *note, *velocity);
                    active_notes.push((*channel, *note));
                }
                SmafEvent::MidiNoteOff { channel, note, velocity } => {
                    sink.midi_note_off(*channel, *note, *velocity);
                    active_notes.retain(|(c, n)| !(*c == *channel && *n == *note));
                }
                SmafEvent::MidiProgramChange { channel, program } => {
                    sink.midi_program_change(*channel, *program);
                }
                SmafEvent::MidiControlChange { channel, control, value } => {
                    sink.midi_control_change(*channel, *control, *value);
                }
                SmafEvent::End => {}
            }
        }
    }
}
