use alloc::collections::BTreeMap;
use core::time::Duration;

use smaf::Smaf;
use smaf_player::{play_smaf, AudioBackend};

use crate::{audio_sink::AudioSink, System};

struct AudioBackendImpl {
    system: System,
    sink: Box<dyn AudioSink>,
}

#[async_trait::async_trait(?Send)]
impl AudioBackend for AudioBackendImpl {
    fn play_wave(&self, channel: u8, sampling_rate: u32, wave_data: &[i16]) {
        self.sink.play_wave(channel, sampling_rate, wave_data);
    }

    fn midi_note_on(&self, _channel_id: u8, _note: u8, _velocity: u8) {
        // TODO
    }

    fn midi_note_off(&self, _channel_id: u8, _note: u8, _velocity: u8) {
        // TODO
    }

    fn midi_program_change(&self, _channel_id: u8, _program: u8) {
        // TODO
    }

    fn midi_control_change(&self, _channel_id: u8, _control: u8, _value: u8) {
        // TODO
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

enum AudioFile {
    Smaf(Vec<u8>),
}

pub struct Audio {
    backend: AudioBackendImpl,
    files: BTreeMap<AudioHandle, AudioFile>,
    last_audio_handle: AudioHandle,
}

impl Audio {
    pub fn new(sink: Box<dyn AudioSink>, system: System) -> Self {
        Self {
            backend: AudioBackendImpl { sink, system },
            files: BTreeMap::new(),
            last_audio_handle: 0,
        }
    }

    pub fn load_smaf(&mut self, data: &[u8]) -> anyhow::Result<AudioHandle> {
        let audio_handle = self.last_audio_handle;

        self.last_audio_handle += 1;
        self.files.insert(audio_handle, AudioFile::Smaf(data.to_vec()));

        Ok(audio_handle)
    }

    pub async fn play(&self, audio_handle: AudioHandle) -> anyhow::Result<()> {
        match self.files.get(&audio_handle) {
            Some(AudioFile::Smaf(data)) => {
                let smaf = Smaf::parse(data)?;
                play_smaf(&smaf, &self.backend).await;
            }
            None => {
                anyhow::bail!("audio handle not found");
            }
        }

        Ok(())
    }
}
