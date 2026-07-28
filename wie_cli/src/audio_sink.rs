#[cfg(target_arch = "wasm32")]
use std::sync::Mutex;
use std::sync::mpsc::Sender;

#[cfg(not(target_arch = "wasm32"))]
use crate::midi_worker::MidiWorker;

pub struct AudioSink {
    #[cfg(not(target_arch = "wasm32"))]
    midi_worker: Option<MidiWorker>,
    #[cfg(target_arch = "wasm32")]
    midi_out: Option<Mutex<midir::MidiOutputConnection>>,
    audio_tx: Sender<(u8, u32, Vec<i16>)>,
}

impl AudioSink {
    pub fn new(midi_out: Option<midir::MidiOutputConnection>, audio_tx: Sender<(u8, u32, Vec<i16>)>) -> Self {
        Self {
            // Desktop targets share the same scheduler. The browser keeps its
            // existing Web MIDI path, whose timing is managed by the web host.
            #[cfg(not(target_arch = "wasm32"))]
            midi_worker: midi_out.and_then(MidiWorker::new),
            #[cfg(target_arch = "wasm32")]
            midi_out: midi_out.map(Mutex::new),
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
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(worker) = self.midi_worker.as_ref() {
            worker.send(wie_backend::MidiEvent::NoteOn {
                channel: channel_id,
                note,
                velocity,
            });
        }
        #[cfg(target_arch = "wasm32")]
        if let Some(connection) = self.midi_out.as_ref() {
            let _ = connection.lock().unwrap().send(&[0x90 | channel_id, note, velocity]);
        }
    }

    fn midi_note_off(&self, channel_id: u8, note: u8, velocity: u8) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(worker) = self.midi_worker.as_ref() {
            worker.send(wie_backend::MidiEvent::NoteOff {
                channel: channel_id,
                note,
                velocity,
            });
        }
        #[cfg(target_arch = "wasm32")]
        if let Some(connection) = self.midi_out.as_ref() {
            let _ = connection.lock().unwrap().send(&[0x80 | channel_id, note, velocity]);
        }
    }

    fn midi_control_change(&self, channel_id: u8, control: u8, value: u8) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(worker) = self.midi_worker.as_ref() {
            worker.send(wie_backend::MidiEvent::ControlChange {
                channel: channel_id,
                control,
                value,
            });
        }
        #[cfg(target_arch = "wasm32")]
        if let Some(connection) = self.midi_out.as_ref() {
            let _ = connection.lock().unwrap().send(&[0xb0 | channel_id, control, value]);
        }
    }

    fn midi_program_change(&self, channel_id: u8, program: u8) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(worker) = self.midi_worker.as_ref() {
            worker.send(wie_backend::MidiEvent::ProgramChange {
                channel: channel_id,
                program,
            });
        }
        #[cfg(target_arch = "wasm32")]
        if let Some(connection) = self.midi_out.as_ref() {
            let _ = connection.lock().unwrap().send(&[0xc0 | channel_id, program]);
        }
    }

    fn midi_pitch_bend(&self, channel_id: u8, value: u16) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(worker) = self.midi_worker.as_ref() {
            worker.send(wie_backend::MidiEvent::PitchBend { channel: channel_id, value });
        }
        #[cfg(target_arch = "wasm32")]
        if let Some(connection) = self.midi_out.as_ref() {
            let _ = connection
                .lock()
                .unwrap()
                .send(&[0xe0 | channel_id, (value & 0x7f) as u8, ((value >> 7) & 0x7f) as u8]);
        }
    }

    fn midi_sysex(&self, data: &[u8]) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(worker) = self.midi_worker.as_ref() {
            worker.send(wie_backend::MidiEvent::SysEx(data.to_vec()));
        }
        #[cfg(target_arch = "wasm32")]
        if let Some(connection) = self.midi_out.as_ref() {
            let _ = connection.lock().unwrap().send(data);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn schedule_midi(&self, playback_id: u64, events: &[wie_backend::ScheduledMidiEvent], duration_ms: u64, repeat: bool) -> bool {
        self.midi_worker
            .as_ref()
            .is_some_and(|worker| worker.schedule(playback_id, events, duration_ms, repeat))
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn stop_scheduled_midi(&self, playback_id: u64) {
        if let Some(worker) = self.midi_worker.as_ref() {
            worker.stop(playback_id);
        }
    }
}
