use alloc::boxed::Box;

use wie_backend::{AudioSink, Platform};

pub struct TestPlatform;

impl Platform for TestPlatform {
    fn screen(&mut self) -> &mut dyn wie_backend::Screen {
        todo!()
    }

    fn now(&self) -> wie_backend::Instant {
        todo!()
    }

    fn database_repository(&self) -> &dyn wie_backend::DatabaseRepository {
        todo!()
    }

    fn audio_sink(&self) -> Box<dyn AudioSink> {
        Box::new(TestAudioSink)
    }

    fn write_stdout(&self, _buf: &[u8]) {}
}

struct TestAudioSink;

impl AudioSink for TestAudioSink {
    fn play_wave(&self, _channel: u8, _sampling_rate: u32, _wave_data: &[i16]) {
        todo!()
    }

    fn midi_note_on(&self, _channel_id: u8, _note: u8, _velocity: u8) {
        todo!()
    }

    fn midi_note_off(&self, _channel_id: u8, _note: u8, _velocity: u8) {
        todo!()
    }

    fn midi_program_change(&self, _channel_id: u8, _program: u8) {
        todo!()
    }

    fn midi_control_change(&self, _channel_id: u8, _control: u8, _value: u8) {
        todo!()
    }
}
