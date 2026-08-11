use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    num::NonZero,
    sync::{Arc, mpsc::Receiver, mpsc::RecvTimeoutError, mpsc::Sender},
    time::{Duration, Instant},
};

use midir::{MidiOutput, MidiOutputConnection};
use rodio::{DeviceSinkBuilder, Player, buffer::SamplesBuffer, conversions::SampleTypeConverter};
use wie_backend::{AudioCommand, AudioEventData, AudioHandle, AudioSequence, TimedAudioEvent};

use crate::select_midi_output_index;

pub struct AudioSink {
    tx: Sender<AudioCommand>,
}

impl AudioSink {
    pub fn new(tx: Sender<AudioCommand>) -> Self {
        Self { tx }
    }
}

impl wie_backend::AudioSink for AudioSink {
    fn send(&self, command: AudioCommand) {
        if self.tx.send(command).is_err() {
            tracing::warn!("Audio worker is unavailable");
        }
    }
}

struct Playback {
    sequence: Arc<AudioSequence>,
    repeat: bool,
    started_at: Instant,
    next_event: usize,
    active_notes: BTreeSet<(u8, u8)>,
    used_channels: BTreeSet<u8>,
}

impl Playback {
    fn new(sequence: Arc<AudioSequence>, repeat: bool) -> Self {
        Self {
            sequence,
            repeat,
            started_at: Instant::now(),
            next_event: 0,
            active_notes: BTreeSet::new(),
            used_channels: BTreeSet::new(),
        }
    }

    fn next_deadline(&self) -> Instant {
        let time = self
            .sequence
            .events
            .get(self.next_event)
            .map_or(self.sequence.duration, |event| event.time);
        self.started_at + Duration::from_millis(time)
    }
}

pub fn run(rx: Receiver<AudioCommand>, midi_device: Option<usize>) {
    let mut midi_out = match (|| {
        let midi_out = MidiOutput::new("wie_cli")?;
        let midi_ports = midi_out.ports();
        let port_index = select_midi_output_index(midi_ports.len(), midi_device).ok_or_else(|| anyhow::anyhow!("No MIDI output port"))?;
        let out_port = &midi_ports[port_index];
        let port_name = midi_out.port_name(out_port).unwrap_or_else(|_| "<unknown>".to_string());

        if let Some(requested) = midi_device
            && requested != port_index
        {
            tracing::warn!(requested, fallback = %port_name, "Requested MIDI output index is out of range; using default");
        }

        let connection = midi_out.connect(out_port, "wie_cli")?;
        tracing::info!(port = %port_name, "Using MIDI output");
        Ok::<_, Box<dyn Error>>(connection)
    })() {
        Ok(connection) => Some(connection),
        Err(error) => {
            tracing::warn!(%error, "MIDI output is unavailable");
            None
        }
    };

    let output_sink = match DeviceSinkBuilder::open_default_sink() {
        Ok(output_sink) => Some(output_sink),
        Err(error) => {
            tracing::warn!(%error, "PCM output is unavailable");
            None
        }
    };
    let player = output_sink.as_ref().map(|output_sink| Player::connect_new(output_sink.mixer()));
    let mut playbacks = BTreeMap::new();

    loop {
        let command = if let Some(deadline) = playbacks.values().map(Playback::next_deadline).min() {
            match rx.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
                Ok(command) => Some(command),
                Err(RecvTimeoutError::Timeout) => None,
                Err(RecvTimeoutError::Disconnected) => break,
            }
        } else {
            match rx.recv() {
                Ok(command) => Some(command),
                Err(_) => break,
            }
        };

        if let Some(command) = command {
            match command {
                AudioCommand::Play { handle, sequence, repeat } => {
                    if let Some(mut playback) = playbacks.remove(&handle) {
                        cleanup(&mut midi_out, &mut playback);
                    }
                    playbacks.insert(handle, Playback::new(sequence, repeat));
                }
                AudioCommand::Stop { handle } => {
                    if let Some(mut playback) = playbacks.remove(&handle) {
                        cleanup(&mut midi_out, &mut playback);
                    }
                }
            }
            continue;
        }

        let now = Instant::now();
        let handles: Vec<AudioHandle> = playbacks.keys().copied().collect();
        for handle in handles {
            let playback = playbacks.get_mut(&handle).unwrap();

            while let Some(event) = playback.sequence.events.get(playback.next_event) {
                if playback.started_at + Duration::from_millis(event.time) > now {
                    break;
                }

                play_event(
                    &mut midi_out,
                    player.as_ref(),
                    event,
                    &mut playback.active_notes,
                    &mut playback.used_channels,
                );
                playback.next_event += 1;
            }

            if playback.next_event == playback.sequence.events.len() && playback.started_at + Duration::from_millis(playback.sequence.duration) <= now
            {
                cleanup(&mut midi_out, playback);

                if playback.repeat && playback.sequence.duration != 0 {
                    playback.started_at = now;
                    playback.next_event = 0;
                } else {
                    playbacks.remove(&handle);
                }
            }
        }
    }

    for playback in playbacks.values_mut() {
        cleanup(&mut midi_out, playback);
    }
}

fn play_event(
    midi_out: &mut Option<MidiOutputConnection>,
    player: Option<&Player>,
    event: &TimedAudioEvent,
    active_notes: &mut BTreeSet<(u8, u8)>,
    used_channels: &mut BTreeSet<u8>,
) {
    match &event.data {
        AudioEventData::Midi(data) => {
            if let Some(status) = data.first().copied()
                && (0x80..0xf0).contains(&status)
            {
                let channel = status & 0x0f;
                used_channels.insert(channel);

                if let Some(note) = data.get(1).copied() {
                    match status & 0xf0 {
                        0x80 => {
                            active_notes.remove(&(channel, note));
                        }
                        0x90 if data.get(2).copied().unwrap_or(0) == 0 => {
                            active_notes.remove(&(channel, note));
                        }
                        0x90 => {
                            active_notes.insert((channel, note));
                        }
                        _ => {}
                    }
                }
            }

            if let Some(midi_out) = midi_out
                && let Err(error) = midi_out.send(data)
            {
                tracing::warn!(%error, "Failed to send MIDI event");
            }
        }
        AudioEventData::Wave {
            channels,
            sampling_rate,
            samples,
        } => {
            let (Some(player), Some(channels), Some(sampling_rate)) = (player, NonZero::new((*channels).into()), NonZero::new(*sampling_rate)) else {
                return;
            };

            player.append(SamplesBuffer::new(
                channels,
                sampling_rate,
                SampleTypeConverter::new(samples.iter().copied()).collect::<Vec<_>>(),
            ));
        }
    }
}

fn cleanup(midi_out: &mut Option<MidiOutputConnection>, playback: &mut Playback) {
    let Some(midi_out) = midi_out else {
        playback.active_notes.clear();
        playback.used_channels.clear();
        return;
    };

    for (channel, note) in &playback.active_notes {
        if let Err(error) = midi_out.send(&[0x80 | channel, *note, 0]) {
            tracing::warn!(%error, "Failed to stop MIDI note");
        }
    }
    for channel in &playback.used_channels {
        for control in [64, 120, 123] {
            if let Err(error) = midi_out.send(&[0xb0 | channel, control, 0]) {
                tracing::warn!(%error, "Failed to reset MIDI channel");
            }
        }
    }

    playback.active_notes.clear();
    playback.used_channels.clear();
}
