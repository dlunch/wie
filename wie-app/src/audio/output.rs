use std::{collections::BTreeMap, num::NonZero};

use anyhow::{Context, Result};
use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player, buffer::SamplesBuffer, conversions::SampleTypeConverter, mixer::Mixer};
use tauri::AppHandle;
use wie_backend::{AudioEventData, AudioHandle, AudioSequence};

use super::Warning;
#[cfg(target_os = "android")]
use super::android::Midi;
#[cfg(any(target_os = "linux", target_os = "windows"))]
use super::midi::Midi;

pub(super) struct Output {
    midi: Midi,
    pcm: Pcm,
    _device: MixerDeviceSink,
}

struct Pcm {
    mixer: Mixer,
    players: BTreeMap<AudioHandle, Vec<Player>>,
    volume: f32,
}

impl Pcm {
    fn play(&mut self, handle: AudioHandle, channels: u8, sampling_rate: u32, samples: &[i16]) -> Result<()> {
        let channels = NonZero::new(u16::from(channels)).context("PCM channel count is zero")?;
        let rate = NonZero::new(sampling_rate).context("PCM sample rate is zero")?;
        let player = Player::connect_new(&self.mixer);
        player.set_volume(self.volume);
        player.append(SamplesBuffer::new(
            channels,
            rate,
            SampleTypeConverter::new(samples.iter().copied()).collect::<Vec<_>>(),
        ));
        self.players.entry(handle).or_default().push(player);
        Ok(())
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
        for player in self.players.values().flatten() {
            player.set_volume(volume);
        }
    }

    #[cfg(any(mobile, test))]
    fn pause(&self, paused: bool) {
        for player in self.players.values().flatten() {
            if paused {
                player.pause();
            } else {
                player.play();
            }
        }
    }
}

impl Output {
    pub fn new(app: &AppHandle, midi_volume: f32, pcm_volume: f32, warning: &Warning) -> Result<Self> {
        #[cfg(target_os = "android")]
        let midi = Midi::new(app, midi_volume, warning)?;
        #[cfg(any(target_os = "linux", target_os = "windows"))]
        let midi = {
            let _ = app;
            Midi::new(midi_volume, warning)
        };
        let device = DeviceSinkBuilder::open_default_sink()?;
        let pcm = Pcm {
            mixer: device.mixer().clone(),
            players: BTreeMap::new(),
            volume: pcm_volume,
        };
        Ok(Self { midi, pcm, _device: device })
    }

    pub fn start(&mut self, handle: AudioHandle, sequence: &AudioSequence, repeat: bool) -> Result<()> {
        self.midi.start(handle, sequence, repeat)
    }

    pub fn event(&mut self, handle: AudioHandle, event: &AudioEventData) -> Result<()> {
        match event {
            AudioEventData::Midi(data) => self.midi.event(handle, data)?,
            AudioEventData::Wave {
                channels,
                sampling_rate,
                samples,
            } => self.pcm.play(handle, *channels, *sampling_rate, samples)?,
        }
        Ok(())
    }

    pub fn finish(&mut self, handle: AudioHandle) -> Result<()> {
        self.midi.finish(handle)
    }

    pub fn stop(&mut self, handle: AudioHandle) -> Result<()> {
        self.pcm.players.remove(&handle);
        self.midi.stop(handle)
    }

    pub fn set_volumes(&mut self, midi: f32, pcm: f32) -> Result<()> {
        self.pcm.set_volume(pcm);
        self.midi.set_volume(midi)
    }

    #[cfg(mobile)]
    pub fn pause(&mut self) -> Result<()> {
        self.pcm.pause(true);
        #[cfg(target_os = "android")]
        self.midi.pause()?;
        Ok(())
    }

    #[cfg(mobile)]
    pub fn resume(&mut self) -> Result<()> {
        #[cfg(target_os = "android")]
        self.midi.resume()?;
        self.pcm.pause(false);
        Ok(())
    }

    pub fn has_tails(&self) -> bool {
        !self.pcm.players.is_empty()
    }

    pub fn reap(&mut self) {
        self.pcm.players.retain(|_, players| {
            players.retain(|player| !player.empty());
            !players.is_empty()
        });
        self.midi.reap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rodio::mixer::mixer;

    #[test]
    fn waves_overlap_and_handle_drop_volume_and_pause_control_the_mixed_output() {
        let (mixer, mut output) = mixer(NonZero::new(1).unwrap(), NonZero::new(1000).unwrap());
        let mut pcm = Pcm {
            mixer,
            players: BTreeMap::new(),
            volume: 1.0,
        };
        pcm.play(1, 1, 1000, &vec![8192; 1000]).unwrap();
        pcm.play(1, 1, 1000, &vec![8192; 1000]).unwrap();
        pcm.play(2, 1, 1000, &vec![4096; 1000]).unwrap();
        let first_sound = output.by_ref().take(100).find(|sample| *sample != 0.0);
        assert_eq!(first_sound, Some(0.625));
        pcm.players.remove(&1);
        for _ in 0..10 {
            output.next();
        }
        assert_eq!(output.next(), Some(0.125));
        pcm.set_volume(0.5);
        for _ in 0..10 {
            output.next();
        }
        assert_eq!(output.next(), Some(0.0625));
        pcm.pause(true);
        for _ in 0..10 {
            output.next();
        }
        assert_eq!(output.next(), Some(0.0));
        pcm.pause(false);
        for _ in 0..10 {
            output.next();
        }
        assert_eq!(output.next(), Some(0.0625));
        drop(pcm);
        for _ in 0..20 {
            output.next();
        }
        assert_eq!(output.next(), None);
    }
}
