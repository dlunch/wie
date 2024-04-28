use alloc::{collections::BTreeMap, sync::Arc};
use core::time::Duration;

use smaf_player::{AudioBackend, SmafPlayer};

use crate::{audio_sink::AudioSink, System};

struct AudioBackendImpl {
    system: System,
    sink: Box<dyn AudioSink>,
}

#[async_trait::async_trait]
impl AudioBackend for AudioBackendImpl {
    fn play_wave(&self, channel: u8, sampling_rate: u32, wave_data: &[i16]) {
        self.sink.play_wave(channel, sampling_rate, wave_data);
    }

    fn midi_note_on(&self, channel_id: u8, note: u8, velocity: u8) {
        self.sink.midi_note_on(channel_id, note, velocity);
    }

    fn midi_note_off(&self, channel_id: u8, note: u8, velocity: u8) {
        self.sink.midi_note_off(channel_id, note, velocity);
    }

    fn midi_program_change(&self, channel_id: u8, program: u8) {
        self.sink.midi_program_change(channel_id, program);
    }

    fn midi_control_change(&self, channel_id: u8, control: u8, value: u8) {
        self.sink.midi_control_change(channel_id, control, value);
    }

    async fn sleep(&self, duration: Duration) {
        let now = self.system.platform().now();
        let end = now + duration.as_millis() as _;

        self.system.clone().sleep(end).await
    }

    fn now_millis(&self) -> u64 {
        self.system.platform().now().raw()
    }
}

pub type AudioHandle = u32;
#[derive(Debug)]
pub enum AudioError {
    InvalidHandle,
    InvalidAudio,
}

enum AudioFile {
    Smaf(SmafPlayer),
}

pub struct Audio {
    system: System,
    backend: Arc<AudioBackendImpl>,
    files: BTreeMap<AudioHandle, AudioFile>,
    last_audio_handle: AudioHandle,
}

impl Audio {
    pub fn new(sink: Box<dyn AudioSink>, system: System) -> Self {
        Self {
            system: system.clone(),
            backend: Arc::new(AudioBackendImpl { sink, system }),
            files: BTreeMap::new(),
            last_audio_handle: 0,
        }
    }

    pub fn load_smaf(&mut self, data: &[u8]) -> Result<AudioHandle, AudioError> {
        let audio_handle = self.last_audio_handle;

        self.last_audio_handle += 1;
        self.files.insert(audio_handle, AudioFile::Smaf(SmafPlayer::new(data.to_vec())));

        Ok(audio_handle)
    }

    pub async fn play(&self, audio_handle: AudioHandle) -> Result<(), AudioError> {
        match self.files.get(&audio_handle) {
            Some(AudioFile::Smaf(player)) => {
                let player_clone = player.clone();
                let backend = self.backend.clone();

                self.system.clone().spawn(move || async move {
                    player_clone.play(&*backend).await;

                    Ok::<_, AudioError>(())
                });
            }
            None => return Err(AudioError::InvalidHandle),
        }

        Ok(())
    }
}
