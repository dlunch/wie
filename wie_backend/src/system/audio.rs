use alloc::collections::BTreeMap;

use smaf::Smaf;
use smaf_player::{play_smaf, AudioBackend};

use crate::audio_sink::AudioSink;

struct AudioBackendImpl {
    sink: Box<dyn AudioSink>,
}

impl AudioBackend for AudioBackendImpl {
    fn play_wave(&self, channel: u8, sampling_rate: u32, wave_data: &[i16]) {
        self.sink.play_wave(channel, sampling_rate, wave_data);
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
    pub fn new(sink: Box<dyn AudioSink>) -> Self {
        Self {
            backend: AudioBackendImpl { sink },
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

    pub fn play(&self, audio_handle: AudioHandle) -> anyhow::Result<()> {
        match self.files.get(&audio_handle) {
            Some(AudioFile::Smaf(data)) => {
                let smaf = Smaf::parse(data)?;
                play_smaf(&smaf, &self.backend);
            }
            None => {
                anyhow::bail!("audio handle not found");
            }
        }

        Ok(())
    }
}
