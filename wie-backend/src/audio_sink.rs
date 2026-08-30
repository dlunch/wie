use alloc::{sync::Arc, vec::Vec};

pub type AudioHandle = u32;

#[derive(Clone, Debug, PartialEq)]
pub enum AudioCommand {
    Play {
        handle: AudioHandle,
        sequence: Arc<AudioSequence>,
        repeat: bool,
    },
    Stop {
        handle: AudioHandle,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct AudioSequence {
    pub duration: u64,
    pub events: Vec<TimedAudioEvent>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimedAudioEvent {
    pub time: u64,
    pub data: AudioEventData,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AudioEventData {
    Midi(Vec<u8>),
    Wave { channels: u8, sampling_rate: u32, samples: Vec<i16> },
}

pub trait AudioSink: Sync + Send {
    fn send(&self, command: AudioCommand);
}
