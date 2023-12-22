use alloc::collections::BTreeMap;

use rodio::{buffer::SamplesBuffer, OutputStream, Sink};

use smaf::Smaf;
use smaf_player::{play_smaf, AudioBackend};

struct AudioBackendImpl;

impl AudioBackend for AudioBackendImpl {
    fn play_wave(&self, channel: u8, sampling_rate: u32, wave_data: &[i16]) {
        let buffer = SamplesBuffer::new(channel as _, sampling_rate as _, wave_data);

        let (_output_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        sink.append(buffer);
    }
}

pub type AudioHandle = u32;

enum AudioFile {
    Smaf(Vec<u8>),
}

#[derive(Default)]
pub struct Audio {
    files: BTreeMap<AudioHandle, AudioFile>,
    last_audio_handle: AudioHandle,
}

impl Audio {
    pub fn new() -> Self {
        Self {
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
                play_smaf(&smaf, &AudioBackendImpl);
            }
            None => {
                anyhow::bail!("audio handle not found");
            }
        }

        Ok(())
    }
}
