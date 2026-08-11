use js_sys::{Array, Int16Array, Uint8Array};
use wasm_bindgen::prelude::*;

use wie_backend::{AudioCommand, AudioEventData};

#[wasm_bindgen(module = "midi.ts")]
extern "C" {
    type AudioPlayer;

    #[wasm_bindgen(constructor)]
    fn new() -> AudioPlayer;

    #[wasm_bindgen(method)]
    fn play(this: &AudioPlayer, handle: u32, duration: f64, events: Array, repeat: bool);

    #[wasm_bindgen(method)]
    fn stop(this: &AudioPlayer, handle: u32);

    #[wasm_bindgen(js_name = setPcmVolume)]
    pub fn set_pcm_volume(value: f32);
}

pub struct AudioSink {
    player: AudioPlayer,
}

// The wasm frontend and its JavaScript audio bridge run on one thread.
unsafe impl Sync for AudioSink {}
unsafe impl Send for AudioSink {}

impl AudioSink {
    pub fn new() -> Self {
        Self { player: AudioPlayer::new() }
    }
}

impl wie_backend::AudioSink for AudioSink {
    fn send(&self, command: AudioCommand) {
        match command {
            AudioCommand::Play { handle, sequence, repeat } => {
                let events = Array::new();
                for event in &sequence.events {
                    let value = Array::new();
                    value.push(&JsValue::from_f64(event.time as f64));

                    match &event.data {
                        AudioEventData::Midi(data) => {
                            value.push(&JsValue::from_str("midi"));
                            value.push(Uint8Array::from(data.as_slice()).as_ref());
                        }
                        AudioEventData::Wave {
                            channels,
                            sampling_rate,
                            samples,
                        } => {
                            value.push(&JsValue::from_str("wave"));
                            value.push(&JsValue::from(*channels));
                            value.push(&JsValue::from(*sampling_rate));
                            value.push(Int16Array::from(samples.as_slice()).as_ref());
                        }
                    }

                    events.push(value.as_ref());
                }

                self.player.play(handle, sequence.duration as f64, events, repeat);
            }
            AudioCommand::Stop { handle } => self.player.stop(handle),
        }
    }
}
