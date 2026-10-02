use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result};
use wie_backend::{AudioEventData, AudioHandle, AudioSequence};

use super::Warning;
#[cfg(all(target_os = "linux", not(test)))]
use super::linux::Connection;
#[cfg(all(target_os = "windows", not(test)))]
use super::windows::Connection;
#[cfg(test)]
use tests::Connection;

struct Voice {
    channels: BTreeMap<u8, u8>,
    percussion: BTreeSet<u8>,
    percussion_volume: u8,
    percussion_expression: u8,
    percussion_bank: [u8; 2],
    percussion_program: u8,
    repeat: bool,
}

pub(super) struct Midi {
    connection: Option<Connection>,
    voices: BTreeMap<AudioHandle, Voice>,
    channel_volumes: [u8; 16],
    volume: f32,
}

impl Midi {
    pub fn new(volume: f32, warning: &Warning) -> Self {
        let connection = match Connection::new() {
            Ok(connection) => Some(connection),
            Err(error) => {
                warning(format!("MIDI is unavailable; game and PCM audio will continue: {error:#}"));
                None
            }
        };
        Self {
            connection,
            voices: BTreeMap::new(),
            channel_volumes: [100; 16],
            volume,
        }
    }

    pub fn start(&mut self, handle: AudioHandle, sequence: &AudioSequence, repeat: bool) -> Result<()> {
        if self.connection.is_none() {
            return Ok(());
        }
        let mut used: BTreeSet<u8> = self.voices.values().flat_map(|voice| voice.channels.values().copied()).collect();
        let shared_percussion = used.contains(&9);
        let mut channels = BTreeMap::new();
        for event in &sequence.events {
            if let AudioEventData::Midi(data) = &event.data
                && let Some(status @ 0x80..=0xef) = data.first()
            {
                let source = status & 0xf;
                if channels.contains_key(&source) {
                    continue;
                }
                let target = if source == 9 || !used.contains(&source) {
                    Some(source)
                } else {
                    (0..16).find(|channel| *channel != 9 && !used.contains(channel))
                }
                .context("The system MIDI synthesizer has no free channel for this playback")?;
                channels.insert(source, target);
                used.insert(target);
            }
        }
        for channel in channels.values() {
            if *channel == 9 && shared_percussion {
                continue;
            }
            self.channel_volumes[*channel as usize] = if *channel == 9 { 127 } else { 100 };
            let connection = self.connection.as_mut().unwrap();
            connection.send(&[0xb0 | channel, 121, 0])?;
            connection.send(&[0xb0 | channel, 0, 0])?;
            connection.send(&[0xb0 | channel, 32, 0])?;
            connection.send(&[0xc0 | channel, 0])?;
            connection.send(&[
                0xb0 | channel,
                7,
                (f32::from(self.channel_volumes[*channel as usize]) * self.volume).round() as u8,
            ])?;
        }
        self.voices.insert(
            handle,
            Voice {
                channels,
                percussion: BTreeSet::new(),
                percussion_volume: 100,
                percussion_expression: 127,
                percussion_bank: [0; 2],
                percussion_program: 0,
                repeat: repeat && sequence.duration != 0,
            },
        );
        Ok(())
    }

    pub fn event(&mut self, handle: AudioHandle, data: &[u8]) -> Result<()> {
        let Some(connection) = &mut self.connection else {
            return Ok(());
        };
        // A MIDI start can fail while this handle's PCM is still being scheduled.
        let shared_percussion = self
            .voices
            .iter()
            .any(|(other, voice)| *other != handle && voice.channels.contains_key(&9));
        let Some(voice) = self.voices.get_mut(&handle) else {
            return Ok(());
        };
        let Some(&status) = data.first() else {
            return Ok(());
        };
        if !(0x80..0xf0).contains(&status) {
            connection.send(data)?;
            // A guest SysEx reset can restore device channel gains. Keep host volume authoritative.
            return self.set_volume(self.volume);
        }
        let channel = voice.channels[&(status & 0xf)];
        if channel == 9 {
            match data {
                [_, note, velocity] if status & 0xf0 == 0x90 && *velocity != 0 => {
                    let velocity = (f32::from(*velocity) * f32::from(voice.percussion_volume) * f32::from(voice.percussion_expression)
                        / (127.0 * 127.0))
                        .round() as u8;
                    // Zero-velocity NoteOn is NoteOff, which could silence another owner's note.
                    if velocity == 0 {
                        return Ok(());
                    }
                    voice.percussion.insert(*note);
                    connection.send(&[0xb9, 0, voice.percussion_bank[0]])?;
                    connection.send(&[0xb9, 32, voice.percussion_bank[1]])?;
                    connection.send(&[0xc9, voice.percussion_program])?;
                    return connection.send(&[0x99, *note, velocity]);
                }
                [_, 7, value] if status & 0xf0 == 0xb0 => {
                    voice.percussion_volume = *value;
                    return Ok(());
                }
                [_, 11, value] if status & 0xf0 == 0xb0 => {
                    voice.percussion_expression = *value;
                    return Ok(());
                }
                [_, 0, value] if status & 0xf0 == 0xb0 => {
                    voice.percussion_bank[0] = *value;
                    return Ok(());
                }
                [_, 32, value] if status & 0xf0 == 0xb0 => {
                    voice.percussion_bank[1] = *value;
                    return Ok(());
                }
                [_, program] if status & 0xf0 == 0xc0 => {
                    voice.percussion_program = *program;
                    return Ok(());
                }
                [_, 121, _] if status & 0xf0 == 0xb0 => {
                    voice.percussion_expression = 127;
                    if shared_percussion {
                        return Ok(());
                    }
                }
                [_, note, _] if status & 0xf0 == 0x80 || status & 0xf0 == 0x90 => {
                    voice.percussion.remove(note);
                    if self
                        .voices
                        .iter()
                        .any(|(other, voice)| *other != handle && voice.percussion.contains(note))
                    {
                        return Ok(());
                    }
                }
                [_, 120 | 123, _] if status & 0xf0 == 0xb0 => {
                    let notes = std::mem::take(&mut voice.percussion);
                    if shared_percussion {
                        for note in notes {
                            if !self
                                .voices
                                .iter()
                                .any(|(other, voice)| *other != handle && voice.percussion.contains(&note))
                            {
                                connection.send(&[0x89, note, 0])?;
                            }
                        }
                        return Ok(());
                    }
                }
                _ if shared_percussion && matches!(status & 0xf0, 0xb0..=0xe0) => {
                    // ponytail: one GM drum channel; gain/kit apply to new notes. Other
                    // channel-wide controls need separate synth instances for isolation.
                    return Ok(());
                }
                _ => {}
            }
        }
        let mut message = [0; 3];
        message[..data.len()].copy_from_slice(data);
        message[0] = (status & 0xf0) | channel;
        if let [_, 7, value] = data
            && status & 0xf0 == 0xb0
        {
            self.channel_volumes[channel as usize] = *value;
            message[2] = (f32::from(*value) * self.volume).round() as u8;
        }
        connection.send(&message[..data.len()])
    }

    pub fn finish(&mut self, handle: AudioHandle) -> Result<()> {
        let Some(mut voice) = self.voices.remove(&handle) else {
            return Ok(());
        };
        let result = (|| {
            if let Some(connection) = &mut self.connection {
                for channel in voice.channels.values() {
                    if *channel == 9 && self.voices.values().any(|other| other.channels.contains_key(&9)) {
                        for note in &voice.percussion {
                            if !self.voices.values().any(|other| other.percussion.contains(note)) {
                                connection.send(&[0x89, *note, 0])?;
                            }
                        }
                        continue;
                    }
                    for control in [64, 120, 123] {
                        connection.send(&[0xb0 | channel, control, 0])?;
                    }
                }
            }
            Ok(())
        })();
        if voice.repeat {
            voice.percussion.clear();
            self.voices.insert(handle, voice);
        }
        result
    }

    pub fn stop(&mut self, handle: AudioHandle) -> Result<()> {
        let result = self.finish(handle);
        self.voices.remove(&handle);
        result
    }

    pub fn set_volume(&mut self, volume: f32) -> Result<()> {
        self.volume = volume;
        if let Some(connection) = &mut self.connection {
            for voice in self.voices.values() {
                for channel in voice.channels.values() {
                    connection.send(&[
                        0xb0 | channel,
                        7,
                        (f32::from(self.channel_volumes[*channel as usize]) * volume).round() as u8,
                    ])?;
                }
            }
        }
        Ok(())
    }

    pub fn reap(&mut self) {
        if let Some(connection) = &mut self.connection {
            connection.reap();
        }
    }
}

impl Drop for Midi {
    fn drop(&mut self) {
        while let Some(handle) = self.voices.keys().next().copied() {
            if let Err(error) = self.stop(handle) {
                log::error!("MIDI cleanup failed: {error:#}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wie_backend::TimedAudioEvent;

    // Only the physical MIDI endpoint is replaced; routing and lifecycle run unchanged.
    pub(super) struct Connection {
        pub messages: Vec<Vec<u8>>,
    }

    impl Connection {
        pub fn new() -> Result<Self> {
            anyhow::bail!("No test MIDI endpoint")
        }

        pub fn send(&mut self, data: &[u8]) -> Result<()> {
            self.messages.push(data.to_vec());
            Ok(())
        }

        pub fn reap(&mut self) {}
    }

    #[test]
    fn overlapping_handles_keep_channels_volume_and_stop_isolated() {
        let mut midi = Midi {
            connection: Some(Connection { messages: vec![] }),
            voices: BTreeMap::new(),
            channel_volumes: [100; 16],
            volume: 0.5,
        };
        let sequence = AudioSequence {
            duration: 100,
            events: vec![TimedAudioEvent {
                time: 0,
                data: AudioEventData::Midi(vec![0x90, 60, 100]),
            }],
        };
        midi.start(1, &sequence, true).unwrap();
        midi.start(2, &sequence, false).unwrap();
        midi.event(1, &[0x90, 60, 100]).unwrap();
        midi.event(2, &[0x90, 60, 100]).unwrap();
        midi.event(2, &[0xb0, 7, 80]).unwrap();
        let messages = &midi.connection.as_ref().unwrap().messages;
        assert!(messages.ends_with(&[vec![0x90, 60, 100], vec![0x91, 60, 100], vec![0xb1, 7, 40]]));
        midi.connection.as_mut().unwrap().messages.clear();
        midi.finish(1).unwrap();
        assert_eq!(
            midi.connection.as_ref().unwrap().messages,
            [vec![0xb0, 64, 0], vec![0xb0, 120, 0], vec![0xb0, 123, 0]]
        );
        midi.event(1, &[0x90, 64, 90]).unwrap();
        midi.stop(1).unwrap();
        assert_eq!(midi.voices[&2].channels[&0], 1);
        midi.connection.as_mut().unwrap().messages.clear();
        midi.set_volume(0.25).unwrap();
        assert_eq!(midi.connection.as_ref().unwrap().messages, [vec![0xb1, 7, 20]]);
        midi.finish(2).unwrap();
        assert!(midi.voices.is_empty());

        let percussion = AudioSequence {
            duration: 100,
            events: vec![TimedAudioEvent {
                time: 0,
                data: AudioEventData::Midi(vec![0x99, 36, 100]),
            }],
        };
        midi.start(1, &percussion, true).unwrap();
        assert!(midi.connection.as_ref().unwrap().messages.ends_with(&[vec![0xb9, 7, 32]]));
        midi.connection.as_mut().unwrap().messages.clear();
        midi.event(1, &[0xb9, 7, 80]).unwrap();
        midi.event(1, &[0xb9, 11, 64]).unwrap();
        assert!(midi.connection.as_ref().unwrap().messages.is_empty());
        midi.event(1, &[0x99, 36, 100]).unwrap();
        midi.event(1, &[0x99, 38, 100]).unwrap();
        assert!(midi.connection.as_ref().unwrap().messages.ends_with(&[vec![0x99, 38, 32]]));
        midi.connection.as_mut().unwrap().messages.clear();
        midi.start(2, &percussion, false).unwrap();
        assert!(midi.connection.as_ref().unwrap().messages.is_empty());
        midi.event(2, &[0xb9, 121, 0]).unwrap();
        midi.event(2, &[0xb9, 7, 10]).unwrap();
        midi.event(2, &[0xb9, 11, 64]).unwrap();
        midi.event(2, &[0xb9, 0, 120]).unwrap();
        midi.event(2, &[0xc9, 8]).unwrap();
        assert!(midi.connection.as_ref().unwrap().messages.is_empty());
        midi.event(2, &[0x99, 36, 100]).unwrap();
        assert_eq!(
            midi.connection.as_ref().unwrap().messages,
            [vec![0xb9, 0, 120], vec![0xb9, 32, 0], vec![0xc9, 8], vec![0x99, 36, 4]]
        );
        midi.connection.as_mut().unwrap().messages.clear();
        midi.event(2, &[0xb9, 7, 0]).unwrap();
        midi.event(2, &[0x99, 36, 100]).unwrap();
        midi.event(2, &[0xb9, 7, 10]).unwrap();
        midi.event(2, &[0xb9, 11, 0]).unwrap();
        midi.event(2, &[0x99, 36, 100]).unwrap();
        assert!(midi.connection.as_ref().unwrap().messages.is_empty());
        midi.event(2, &[0xb9, 121, 0]).unwrap();
        midi.event(2, &[0x99, 42, 100]).unwrap();
        assert!(midi.connection.as_ref().unwrap().messages.ends_with(&[vec![0x99, 42, 8]]));
        midi.connection.as_mut().unwrap().messages.clear();
        midi.finish(1).unwrap();
        assert_eq!(midi.connection.as_ref().unwrap().messages, [vec![0x89, 38, 0]]);
        midi.connection.as_mut().unwrap().messages.clear();
        midi.event(1, &[0x99, 36, 100]).unwrap();
        assert_eq!(
            midi.connection.as_ref().unwrap().messages,
            [vec![0xb9, 0, 0], vec![0xb9, 32, 0], vec![0xc9, 0], vec![0x99, 36, 32]]
        );
        midi.connection.as_mut().unwrap().messages.clear();
        midi.set_volume(0.5).unwrap();
        assert_eq!(midi.connection.as_ref().unwrap().messages, [vec![0xb9, 7, 64], vec![0xb9, 7, 64]]);
        midi.connection.as_mut().unwrap().messages.clear();
        midi.event(2, &[0x89, 36, 0]).unwrap();
        assert!(midi.connection.as_ref().unwrap().messages.is_empty());
        midi.event(2, &[0xb9, 123, 0]).unwrap();
        assert_eq!(midi.connection.as_ref().unwrap().messages, [vec![0x89, 42, 0]]);
        midi.connection.as_mut().unwrap().messages.clear();
        midi.stop(2).unwrap();
        assert!(midi.connection.as_ref().unwrap().messages.is_empty());
        midi.stop(1).unwrap();
        assert_eq!(
            midi.connection.as_ref().unwrap().messages,
            [vec![0xb9, 64, 0], vec![0xb9, 120, 0], vec![0xb9, 123, 0]]
        );

        let full = AudioSequence {
            duration: 100,
            events: (0..16)
                .map(|channel| TimedAudioEvent {
                    time: 0,
                    data: AudioEventData::Midi(vec![0x90 | channel, 60, 100]),
                })
                .collect(),
        };
        midi.start(1, &full, true).unwrap();
        assert!(midi.start(2, &sequence, false).is_err());
        midi.connection.as_mut().unwrap().messages.clear();
        midi.event(2, &[0x90, 60, 100]).unwrap();
        midi.event(2, &[0xf0, 0x7e, 0x7f, 9, 1, 0xf7]).unwrap();
        assert!(midi.connection.as_ref().unwrap().messages.is_empty());
        midi.finish(2).unwrap();
        assert_eq!(midi.voices.len(), 1);
    }

    #[test]
    fn missing_synth_warns_once_without_failing_playback_controls() {
        use std::sync::{Arc, Mutex};

        let messages = Arc::new(Mutex::new(Vec::new()));
        let received = messages.clone();
        let warning: Warning = Box::new(move |message| received.lock().unwrap().push(message));
        let mut midi = Midi::new(0.5, &warning);
        let sequence = AudioSequence {
            duration: 100,
            events: vec![TimedAudioEvent {
                time: 0,
                data: AudioEventData::Midi(vec![0x90, 60, 100]),
            }],
        };
        for handle in 0..3 {
            midi.start(handle, &sequence, false).unwrap();
            midi.event(handle, &[0x90, 60, 100]).unwrap();
            midi.finish(handle).unwrap();
            midi.stop(handle).unwrap();
            midi.reap();
        }
        midi.set_volume(0.0).unwrap();
        assert!(midi.voices.is_empty());
        drop(midi);
        let messages = messages.lock().unwrap();
        assert_eq!(messages.len(), 1);
        assert!(messages[0].contains("game and PCM audio will continue"));
    }
}
