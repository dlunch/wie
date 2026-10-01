use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use anyhow::{Context, Result, anyhow};
use block2::RcBlock;
use objc2::{AllocAnyThread, rc::Retained};
use objc2_audio_toolbox::{AudioComponentDescription, kAudioUnitManufacturer_Apple, kAudioUnitType_MusicDevice};
use objc2_avf_audio::{
    AVAudioEngine, AVAudioFormat, AVAudioMixerNode, AVAudioPCMBuffer, AVAudioPlayerNode, AVAudioPlayerNodeCompletionCallbackType,
    AVAudioUnitMIDIInstrument,
};
use objc2_foundation::NSData;
use tauri::AppHandle;
use wie_backend::{AudioEventData, AudioHandle, AudioSequence};

#[cfg(target_os = "macos")]
use objc2_audio_toolbox::kAudioUnitSubType_DLSSynth;
#[cfg(target_os = "ios")]
use {
    objc2_audio_toolbox::{AudioUnitSetProperty, kAudioUnitScope_Global, kAudioUnitSubType_MIDISynth, kMusicDeviceProperty_SoundBankURL},
    objc2_avf_audio::{AVAudioSession, AVAudioSessionCategoryPlayback},
    objc2_foundation::{NSString, NSURL},
    tauri::{Manager, path::BaseDirectory},
};

use super::Warning;

struct Wave {
    node: Retained<AVAudioPlayerNode>,
    completed: Arc<AtomicBool>,
}

pub(super) struct Output {
    engine: Retained<AVAudioEngine>,
    midi_mixer: Retained<AVAudioMixerNode>,
    pcm_mixer: Retained<AVAudioMixerNode>,
    instruments: BTreeMap<AudioHandle, Retained<AVAudioUnitMIDIInstrument>>,
    waves: BTreeMap<AudioHandle, Vec<Wave>>,
    #[cfg(target_os = "ios")]
    soundbank: Retained<NSURL>,
}

impl Output {
    pub fn new(_app: &AppHandle, midi_volume: f32, pcm_volume: f32, _warning: &Warning) -> Result<Self> {
        unsafe {
            #[cfg(target_os = "ios")]
            let soundbank = {
                let path = _app.path().resolve("audio/GeneralUser-GS.sf2", BaseDirectory::Resource)?;
                NSURL::fileURLWithPath(&NSString::from_str(&path.to_string_lossy()))
            };
            let engine = AVAudioEngine::new();
            let midi_mixer = AVAudioMixerNode::new();
            let pcm_mixer = AVAudioMixerNode::new();
            let main_mixer = engine.mainMixerNode();
            engine.attachNode(&midi_mixer);
            engine.attachNode(&pcm_mixer);
            engine.connect_to_format(&midi_mixer, &main_mixer, None);
            engine.connect_to_format(&pcm_mixer, &main_mixer, None);
            midi_mixer.setOutputVolume(midi_volume);
            pcm_mixer.setOutputVolume(pcm_volume);
            let mut output = Self {
                engine,
                midi_mixer,
                pcm_mixer,
                instruments: BTreeMap::new(),
                waves: BTreeMap::new(),
                #[cfg(target_os = "ios")]
                soundbank,
            };
            output.resume()?;
            Ok(output)
        }
    }

    pub fn start(&mut self, handle: AudioHandle, sequence: &AudioSequence, _repeat: bool) -> Result<()> {
        if !sequence.events.iter().any(|event| matches!(event.data, AudioEventData::Midi(_))) {
            return Ok(());
        }
        unsafe {
            let description = AudioComponentDescription {
                componentType: kAudioUnitType_MusicDevice,
                #[cfg(target_os = "macos")]
                componentSubType: kAudioUnitSubType_DLSSynth,
                #[cfg(target_os = "ios")]
                componentSubType: kAudioUnitSubType_MIDISynth,
                componentManufacturer: kAudioUnitManufacturer_Apple,
                componentFlags: 0,
                componentFlagsMask: 0,
            };
            let instrument = AVAudioUnitMIDIInstrument::initWithAudioComponentDescription(AVAudioUnitMIDIInstrument::alloc(), description);
            #[cfg(target_os = "ios")]
            {
                let url = Retained::as_ptr(&self.soundbank);
                let status = AudioUnitSetProperty(
                    instrument.audioUnit(),
                    kMusicDeviceProperty_SoundBankURL,
                    kAudioUnitScope_Global,
                    0,
                    std::ptr::from_ref(&url).cast(),
                    std::mem::size_of_val(&url) as u32,
                );
                anyhow::ensure!(status == 0, "Could not load bundled MIDI soundbank: OSStatus {status}");
            }
            self.engine.attachNode(&instrument);
            self.engine.connect_to_format(&instrument, &self.midi_mixer, None);
            self.instruments.insert(handle, instrument);
        }
        Ok(())
    }

    pub fn event(&mut self, handle: AudioHandle, event: &AudioEventData) -> Result<()> {
        unsafe {
            match event {
                AudioEventData::Midi(data) => {
                    let Some(instrument) = self.instruments.get(&handle) else {
                        return Ok(());
                    };
                    match data.as_slice() {
                        [0xf0, ..] => instrument.sendMIDISysExEvent(&NSData::with_bytes(data)),
                        [status, first, second] => instrument.sendMIDIEvent_data1_data2(*status, *first, *second),
                        [status, first] => instrument.sendMIDIEvent_data1(*status, *first),
                        _ => {}
                    }
                }
                AudioEventData::Wave {
                    channels,
                    sampling_rate,
                    samples,
                } => {
                    let format = AVAudioFormat::initStandardFormatWithSampleRate_channels(
                        AVAudioFormat::alloc(),
                        f64::from(*sampling_rate),
                        u32::from(*channels),
                    )
                    .context("Unsupported PCM format")?;
                    let frames = u32::try_from(samples.len() / usize::from(*channels))?;
                    let buffer = AVAudioPCMBuffer::initWithPCMFormat_frameCapacity(AVAudioPCMBuffer::alloc(), &format, frames)
                        .context("Could not allocate PCM buffer")?;
                    buffer.setFrameLength(frames);
                    let planes = buffer.floatChannelData();
                    for channel in 0..usize::from(*channels) {
                        let plane = (*planes.add(channel)).as_ptr();
                        for frame in 0..frames as usize {
                            *plane.add(frame) = f32::from(samples[frame * usize::from(*channels) + channel]) / 32768.0;
                        }
                    }
                    let node = AVAudioPlayerNode::new();
                    self.engine.attachNode(&node);
                    self.engine.connect_to_format(&node, &self.pcm_mixer, Some(&format));
                    let completed = Arc::new(AtomicBool::new(false));
                    let done = completed.clone();
                    let callback = RcBlock::new(move |_: AVAudioPlayerNodeCompletionCallbackType| {
                        done.store(true, Ordering::Release);
                    });
                    node.scheduleBuffer_completionCallbackType_completionHandler(
                        &buffer,
                        AVAudioPlayerNodeCompletionCallbackType::DataPlayedBack,
                        RcBlock::as_ptr(&callback),
                    );
                    node.play();
                    self.waves.entry(handle).or_default().push(Wave { node, completed });
                }
            }
        }
        Ok(())
    }

    pub fn finish(&mut self, handle: AudioHandle) -> Result<()> {
        if let Some(instrument) = self.instruments.get(&handle) {
            for channel in 0..16 {
                for control in [64, 120, 123] {
                    unsafe {
                        instrument.sendMIDIEvent_data1_data2(0xb0 | channel, control, 0);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn stop(&mut self, handle: AudioHandle) -> Result<()> {
        unsafe {
            if let Some(instrument) = self.instruments.remove(&handle) {
                self.engine.detachNode(&instrument);
            }
            if let Some(waves) = self.waves.remove(&handle) {
                for wave in waves {
                    wave.node.stop();
                    self.engine.detachNode(&wave.node);
                }
            }
        }
        Ok(())
    }

    pub fn set_volumes(&mut self, midi: f32, pcm: f32) -> Result<()> {
        unsafe {
            self.midi_mixer.setOutputVolume(midi);
            self.pcm_mixer.setOutputVolume(pcm);
        }
        Ok(())
    }

    #[cfg(mobile)]
    pub fn pause(&mut self) -> Result<()> {
        unsafe {
            self.engine.pause();
            #[cfg(target_os = "ios")]
            AVAudioSession::sharedInstance()
                .setActive_error(false)
                .map_err(|error| anyhow!("{error}"))?;
        }
        Ok(())
    }

    pub fn resume(&mut self) -> Result<()> {
        unsafe {
            #[cfg(target_os = "ios")]
            {
                let session = AVAudioSession::sharedInstance();
                session
                    .setCategory_error(AVAudioSessionCategoryPlayback.context("Playback audio category is unavailable")?)
                    .map_err(|error| anyhow!("{error}"))?;
                session.setActive_error(true).map_err(|error| anyhow!("{error}"))?;
            }
            self.engine.startAndReturnError().map_err(|error| anyhow!("{error}"))?;
        }
        Ok(())
    }

    pub fn has_tails(&self) -> bool {
        !self.waves.is_empty()
    }

    pub fn reap(&mut self) {
        self.waves.retain(|_, waves| {
            waves.retain(|wave| {
                if wave.completed.load(Ordering::Acquire) {
                    unsafe {
                        wave.node.stop();
                        self.engine.detachNode(&wave.node);
                    }
                    false
                } else {
                    true
                }
            });
            !waves.is_empty()
        });
    }
}

impl Drop for Output {
    fn drop(&mut self) {
        unsafe {
            self.engine.stop();
            for waves in self.waves.values() {
                for wave in waves {
                    wave.node.stop();
                    self.engine.detachNode(&wave.node);
                }
            }
            for instrument in self.instruments.values() {
                self.engine.detachNode(instrument);
            }
            #[cfg(target_os = "ios")]
            if let Err(error) = AVAudioSession::sharedInstance().setActive_error(false) {
                log::error!("Could not deactivate audio session: {error}");
            }
        }
    }
}
