use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
    vec::Vec,
};
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
        let player = match self.files.get(&audio_handle) {
            Some(AudioFile::Smaf(data)) => SmafPlayer::new(data),
            None => return Err(AudioError::InvalidHandle),
        };

        self.stop(audio_handle);

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

        Ok(())
    }

    pub fn stop(&mut self, audio_handle: AudioHandle) {
        if let Some(stop_flag) = self.playing.remove(&audio_handle) {
            stop_flag.store(true, Ordering::Relaxed);
        }
    }

    pub fn close(&mut self, audio_handle: AudioHandle) -> Result<(), AudioError> {
        self.stop(audio_handle);

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
        let mut used_channels: BTreeSet<u8> = BTreeSet::new();

        let start_time = system.platform().now();
        for (time, event) in &self.events {
            if stop_flag.load(Ordering::Relaxed) {
                break;
            }

            let now = system.platform().now();
            if (*time as u64) > now - start_time {
                system.sleep(((*time as u64) - (now - start_time)) as _).await;

                if stop_flag.load(Ordering::Relaxed) {
                    break;
                }
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
                    used_channels.insert(*channel);
                }
                SmafEvent::MidiNoteOff { channel, note, velocity } => {
                    sink.midi_note_off(*channel, *note, *velocity);
                    active_notes.retain(|(c, n)| !(*c == *channel && *n == *note));
                }
                SmafEvent::MidiProgramChange { channel, program } => {
                    sink.midi_program_change(*channel, *program);
                    used_channels.insert(*channel);
                }
                SmafEvent::MidiControlChange { channel, control, value } => {
                    sink.midi_control_change(*channel, *control, *value);
                    used_channels.insert(*channel);
                }
                SmafEvent::End => {}
            }
        }

        for (channel, note) in &active_notes {
            sink.midi_note_off(*channel, *note, 0);
        }

        // Release sustain and force any lingering voices off on every channel
        // this track touched. Tracks that set sustain pedal (CC 64) or use
        // long release envelopes (e.g. drum voices) otherwise keep ringing
        // after note_off.
        for channel in &used_channels {
            sink.midi_control_change(*channel, 64, 0); // sustain off
            sink.midi_control_change(*channel, 120, 0); // all sound off
            sink.midi_control_change(*channel, 123, 0); // all notes off
        }
    }
}
