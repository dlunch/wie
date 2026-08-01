use alloc::{sync::Arc, vec::Vec};
use core::num::NonZero;

use rodio::{Player, buffer::SamplesBuffer, conversions::SampleTypeConverter};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "midi.ts")]
extern "C" {
    type MidiPlayer;

    #[wasm_bindgen(constructor)]
    fn new() -> MidiPlayer;

    #[wasm_bindgen(method)]
    fn note_on(this: &MidiPlayer, channel_id: u8, note: u8, velocity: u8);
    #[wasm_bindgen(method)]
    fn note_off(this: &MidiPlayer, channel_id: u8, note: u8, velocity: u8);
    #[wasm_bindgen(method)]
    fn control_change(this: &MidiPlayer, channel_id: u8, control: u8, value: u8);
    #[wasm_bindgen(method)]
    fn program_change(this: &MidiPlayer, channel_id: u8, program: u8);
}

pub struct AudioSink {
    midi_player: MidiPlayer,
    sink: Arc<Player>,
}

// XXX we're on single thread
unsafe impl Sync for AudioSink {}
unsafe impl Send for AudioSink {}

impl AudioSink {
    pub fn new(sink: Arc<Player>) -> Self {
        Self {
            midi_player: MidiPlayer::new(),
            sink,
        }
    }
}

impl wie_backend::AudioSink for AudioSink {
    fn play_wave(&self, channel: u8, sampling_rate: u32, wave_data: &[i16]) {
        let Some(channel) = NonZero::new(channel as u16) else {
            return;
        };
        let Some(sampling_rate) = NonZero::new(sampling_rate) else {
            return;
        };

        let buffer = SamplesBuffer::new(
            channel,
            sampling_rate,
            SampleTypeConverter::new(wave_data.iter().copied()).collect::<Vec<_>>(),
        );

        self.sink.append(buffer);
    }

    fn midi_note_on(&self, channel_id: u8, note: u8, velocity: u8) {
        self.midi_player.note_on(channel_id, note, velocity);
    }

    fn midi_note_off(&self, channel_id: u8, note: u8, velocity: u8) {
        self.midi_player.note_off(channel_id, note, velocity);
    }

    fn midi_control_change(&self, channel_id: u8, control: u8, value: u8) {
        self.midi_player.control_change(channel_id, control, value);
    }

    fn midi_program_change(&self, channel_id: u8, program: u8) {
        self.midi_player.program_change(channel_id, program);
    }

    fn midi_pitch_bend(&self, _channel_id: u8, _value: u16) {
        // TODO
    }

    fn midi_sysex(&self, _data: &[u8]) {
        // TODO
    }
}
