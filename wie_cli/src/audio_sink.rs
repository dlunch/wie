use std::sync::{mpsc::Sender, Mutex};

use midir::MidiOutputConnection;

pub struct AudioSink {
    midi_out: Mutex<MidiOutputConnection>,
    audio_tx: Sender<(u8, u32, Vec<i16>)>,
}

impl AudioSink {
    pub fn new(midi_out: MidiOutputConnection, audio_tx: Sender<(u8, u32, Vec<i16>)>) -> Self {
        Self {
            midi_out: Mutex::new(midi_out),
            audio_tx,
        }
    }
}

// XXX wasm32 is single-threaded anyway
#[cfg(target_arch = "wasm32")]
unsafe impl Sync for AudioSink {}
#[cfg(target_arch = "wasm32")]
unsafe impl Send for AudioSink {}

impl wie_backend::AudioSink for AudioSink {
    fn play_wave(&self, channel: u8, sampling_rate: u32, wave_data: &[i16]) {
        self.audio_tx.send((channel, sampling_rate, wave_data.to_vec())).unwrap();
    }

    fn midi_note_on(&self, channel_id: u8, note: u8, velocity: u8) {
        self.midi_out.lock().unwrap().send(&[0x90 | channel_id, note, velocity]).unwrap();
    }

    fn midi_note_off(&self, channel_id: u8, note: u8, velocity: u8) {
        self.midi_out.lock().unwrap().send(&[0x80 | channel_id, note, velocity]).unwrap();
    }

    fn midi_control_change(&self, channel_id: u8, control: u8, value: u8) {
        self.midi_out.lock().unwrap().send(&[0xB0 | channel_id, control, value]).unwrap()
    }

    fn midi_program_change(&self, channel_id: u8, program: u8) {
        self.midi_out.lock().unwrap().send(&[0xC0 | channel_id, program]).unwrap()
    }
}
