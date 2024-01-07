use wie_backend::{AudioSink, Platform, System};
use wie_core_jvm::JvmCore;

struct TestPlatform;

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
}

struct TestAudioSink;

impl AudioSink for TestAudioSink {
    fn play_wave(&self, _channel: u8, _sampling_rate: u32, _wave_data: &[i16]) {
        todo!()
    }
}

pub fn test_core() -> JvmCore {
    let system_handle = System::new(Box::new(TestPlatform)).handle();

    JvmCore::new(&system_handle)
}
