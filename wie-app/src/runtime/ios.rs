use std::{
    ptr::NonNull,
    sync::{Arc, atomic::Ordering, mpsc::Receiver},
};

use anyhow::{Context, Error, Result, ensure};
use block2::RcBlock;
use futures::channel::oneshot;
use midly::live::LiveEvent;
use objc2::{
    MainThreadMarker,
    rc::Retained,
    runtime::{NSObjectProtocol, ProtocolObject},
};
use objc2_foundation::{NSNotification, NSNotificationCenter};
use objc2_ui_kit::{UIApplication, UIApplicationDidBecomeActiveNotification, UIApplicationState, UIApplicationWillResignActiveNotification};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Resource, State};
use tauri_plugin_haptics::HapticsExt;

use wie_backend::{AudioCommand, AudioEventData, AudioSequence, AudioSink, DatabaseRepository as _, Filesystem, StorageRequest, TimedAudioEvent};

use crate::{audio::Audio, database::DatabaseRepository, filesystem::SqliteFilesystem, store::Store};

use super::{AppState, Clock, Runtime, SessionEvent, SessionWorker};

pub(super) type StartedApp = WebApp;

pub(super) enum Command {
    Volumes(f32, f32),
    Suspend(bool),
    Storage(StorageRequest, oneshot::Sender<Value>),
    Audio(AudioCommand),
    Vibrate(u64, u8),
    Stop,
}

impl Runtime {
    fn send(&self, session_id: u64, command: Command) -> Result<(), String> {
        let sender = self
            .session
            .as_ref()
            .filter(|session| session.id == session_id)
            .and_then(|session| session.commands.as_ref())
            .ok_or("App session has stopped")?;
        sender.send(command).map_err(|_| "App worker has stopped".into())
    }
}

struct Observers {
    center: Retained<NSNotificationCenter>,
    tokens: [Retained<ProtocolObject<dyn NSObjectProtocol>>; 2],
}

// Tokens are opaque and only passed to the thread-safe notification center for removal.
// Their callbacks capture a Send + Sync AppHandle and use AppState's runtime mutex.
unsafe impl Send for Observers {}
unsafe impl Sync for Observers {}

impl Resource for Observers {}

impl Drop for Observers {
    fn drop(&mut self) {
        for token in &self.tokens {
            unsafe { self.center.removeObserver(token.as_ref()) };
        }
    }
}

/// Register on the main thread after AppState has been installed.
pub(crate) fn register(app: &AppHandle) -> Result<()> {
    let main = MainThreadMarker::new().context("iOS lifecycle registration requires the main thread")?;
    let center = NSNotificationCenter::defaultCenter();
    let notifications = unsafe {
        [
            (UIApplicationWillResignActiveNotification, true),
            (UIApplicationDidBecomeActiveNotification, false),
        ]
    };
    let tokens = notifications.map(|(name, suspended)| {
        let app = app.clone();
        let callback = RcBlock::new(move |_: NonNull<NSNotification>| {
            app.state::<AppState>().suspend(suspended);
        });
        // A nil queue delivers synchronously on UIKit's posting thread.
        unsafe { center.addObserverForName_object_queue_usingBlock(Some(name), None, None, &callback) }
    });
    // Tauri explicitly clears resources on exit, releasing the blocks' AppHandles too.
    app.resources_table().add(Observers { center, tokens });
    app.state::<AppState>()
        .suspend(UIApplication::sharedApplication(main).applicationState() != UIApplicationState::Active);
    Ok(())
}

async fn dispatch_storage(request: StorageRequest, store: &Store) -> Value {
    let filesystem = SqliteFilesystem { store: store.clone() };
    let repository = DatabaseRepository { store: store.clone() };
    match request {
        StorageRequest::FileExists { aid, path } => json!(filesystem.exists(&aid, &path).await),
        StorageRequest::FileSize { aid, path } => json!(filesystem.size(&aid, &path).await),
        StorageRequest::FileRead { aid, path, offset, count } => {
            let Some(length) = filesystem.size(&aid, &path).await else {
                return Value::Null;
            };
            // Bound allocation by stored bytes, not an untrusted IPC read count.
            let count = count.min(length.saturating_sub(offset));
            let mut bytes = vec![0; count];
            match filesystem.read(&aid, &path, offset, count, &mut bytes).await {
                Some(read) => {
                    bytes.truncate(read);
                    json!(bytes)
                }
                None => Value::Null,
            }
        }
        StorageRequest::FileWrite { aid, path, offset, data } => {
            json!(filesystem.write(&aid, &path, offset, &data).await)
        }
        StorageRequest::FileTruncate { aid, path, length } => {
            filesystem.truncate(&aid, &path, length).await;
            Value::Null
        }
        StorageRequest::DbOpen { pid, name } => {
            repository.open(&name, &pid).await;
            Value::Null
        }
        StorageRequest::DbExists { pid, name } => json!(repository.exists(&name, &pid).await),
        StorageRequest::DbDelete { pid, name } => json!(repository.delete(&name, &pid).await),
        StorageRequest::DbUsage { pid } => json!(repository.usage(&pid).await),
        StorageRequest::RecordNextId { pid, name } => json!(repository.database(&name, &pid).next_id().await),
        StorageRequest::RecordIds { pid, name } => json!(repository.database(&name, &pid).get_record_ids().await),
        StorageRequest::RecordGet { pid, name, id } => json!(repository.database(&name, &pid).get(id).await),
        StorageRequest::RecordDelete { pid, name, id } => json!(repository.database(&name, &pid).delete(id).await),
        StorageRequest::RecordAdd { pid, name, data } => json!(repository.database(&name, &pid).add(&data).await),
        StorageRequest::RecordSet { pid, name, id, data } => json!(repository.database(&name, &pid).set(id, &data).await),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WebApp {
    pub session_id: u64,
    pub filename: String,
    pub bytes: Vec<u8>,
}

#[tauri::command]
pub(crate) async fn guest_storage(state: State<'_, AppState>, session_id: u64, request: StorageRequest) -> Result<Value, String> {
    let (reply, result) = oneshot::channel();
    state.runtime.lock().unwrap().send(session_id, Command::Storage(request, reply))?;
    result.await.map_err(|_| "App worker stopped before completing storage".into())
}

#[tauri::command]
pub(crate) async fn guest_audio(state: State<'_, AppState>, session_id: u64, command: AudioRequest) -> Result<(), String> {
    let command = command.try_into().map_err(|error: Error| error.to_string())?;
    state.runtime.lock().unwrap().send(session_id, Command::Audio(command))
}

#[tauri::command]
pub(crate) async fn guest_vibrate(state: State<'_, AppState>, session_id: u64, duration_ms: u64, intensity: u8) -> Result<(), String> {
    state.runtime.lock().unwrap().send(session_id, Command::Vibrate(duration_ms, intensity))
}

impl SessionWorker {
    pub(super) fn prepare(self, session_id: u64, filename: String, bytes: Vec<u8>) -> (StartedApp, Self) {
        (WebApp { session_id, filename, bytes }, self)
    }

    pub(super) fn run(self, commands: Receiver<Command>, started: oneshot::Sender<()>) -> Result<()> {
        let warnings = self.events.clone();
        let mut audio = Audio::new(&self.app, self.settings.midi_volume, self.settings.pcm_volume, move |message| {
            let _ = warnings.send(SessionEvent::Warning { message });
        })?;
        let mut clock = Clock::new()?;
        clock.set_paused(self.suspended);
        audio.pause(self.suspended)?;
        self.events.send(SessionEvent::Lifecycle {
            suspended: self.suspended,
            guest_time_ms: clock.now().raw(),
        })?;
        self.initialized.store(true, Ordering::Release);
        let _ = started.send(());
        let sink = audio.sink();
        let mut failure = None;
        while let Ok(command) = commands.recv() {
            let result = match command {
                Command::Storage(request, reply) => {
                    let _ = reply.send(tauri::async_runtime::block_on(dispatch_storage(request, &self.store)));
                    Ok(())
                }
                Command::Audio(command) => {
                    sink.send(command);
                    Ok(())
                }
                Command::Vibrate(duration, intensity) => {
                    if duration != 0 && intensity != 0 {
                        self.app.haptics().vibrate(duration.min(u64::from(u32::MAX)) as u32).map_err(Error::from)
                    } else {
                        Ok(())
                    }
                }
                Command::Volumes(midi, pcm) => audio.set_volumes(midi, pcm),
                Command::Suspend(suspended) => {
                    clock.set_paused(suspended);
                    audio.pause(suspended).and_then(|()| {
                        self.events
                            .send(SessionEvent::Lifecycle {
                                suspended,
                                guest_time_ms: clock.now().raw(),
                            })
                            .map_err(Error::from)
                    })
                }
                Command::Stop => break,
            };
            if let Err(error) = result {
                failure.get_or_insert(error);
                // Close admission, then drain already accepted storage before completion reports the error.
                let _ = self.app.state::<AppState>().stop_app();
            }
        }
        audio.shutdown();
        match failure {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub(crate) enum AudioRequest {
    Play {
        handle: u32,
        duration: u64,
        events: Vec<AudioEventWire>,
        repeat: bool,
    },
    Stop {
        handle: u32,
    },
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub(crate) enum AudioEventWire {
    Midi {
        time: u64,
        data: Vec<u8>,
    },
    Wave {
        time: u64,
        channels: u8,
        sampling_rate: u32,
        samples: Vec<i16>,
    },
}

impl TryFrom<AudioRequest> for AudioCommand {
    type Error = Error;

    fn try_from(request: AudioRequest) -> Result<Self> {
        Ok(match request {
            AudioRequest::Stop { handle } => Self::Stop { handle },
            AudioRequest::Play {
                handle,
                duration,
                events,
                repeat,
            } => {
                let mut previous = 0;
                let events = events
                    .into_iter()
                    .map(|event| {
                        let (time, data) = match event {
                            AudioEventWire::Midi { time, data } => {
                                LiveEvent::parse(&data).context("Invalid MIDI message")?;
                                (time, AudioEventData::Midi(data))
                            }
                            AudioEventWire::Wave {
                                time,
                                channels,
                                sampling_rate,
                                samples,
                            } => {
                                ensure!(channels != 0 && sampling_rate != 0, "Invalid PCM format");
                                ensure!(samples.len().is_multiple_of(usize::from(channels)), "Incomplete PCM frame");
                                (
                                    time,
                                    AudioEventData::Wave {
                                        channels,
                                        sampling_rate,
                                        samples,
                                    },
                                )
                            }
                        };
                        ensure!(
                            time >= previous && time <= duration,
                            "Audio events must be ordered within the sequence duration"
                        );
                        previous = time;
                        Ok(TimedAudioEvent { time, data })
                    })
                    .collect::<Result<_>>()?;
                Self::Play {
                    handle,
                    sequence: Arc::new(AudioSequence { duration, events }),
                    repeat,
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};
    use tempfile::tempdir;

    use super::{AudioRequest, dispatch_storage};
    use crate::store;
    use wie_backend::{AudioCommand, AudioEventData, StorageRequest};

    #[test]
    fn storage_dispatch_preserves_namespaces_empty_values_and_persistence() {
        let root = tempdir().unwrap();
        tauri::async_runtime::block_on(async {
            let store = store::open(root.path()).unwrap();
            for (request, expected) in [
                (json!({"op":"fileRead","aid":"aid","path":"save","offset":0,"count":8}), Value::Null),
                (json!({"op":"fileWrite","aid":"aid","path":"save","offset":2,"data":[7,8]}), json!(2)),
                (json!({"op":"fileRead","aid":"aid","path":"save","offset":1,"count":8}), json!([0, 7, 8])),
                (json!({"op":"fileTruncate","aid":"aid","path":"save","length":0}), Value::Null),
                (json!({"op":"fileRead","aid":"aid","path":"save","offset":0,"count":8}), json!([])),
                (json!({"op":"fileExists","aid":"other","path":"save"}), json!(false)),
                (json!({"op":"dbOpen","pid":"pid","name":"save"}), Value::Null),
                (json!({"op":"recordAdd","pid":"pid","name":"save","data":[]}), json!(1)),
                (json!({"op":"recordGet","pid":"pid","name":"save","id":1}), json!([])),
                (json!({"op":"recordGet","pid":"other","name":"save","id":1}), Value::Null),
                (json!({"op":"dbExists","pid":"other","name":"save"}), json!(false)),
                (json!({"op":"recordSet","pid":"pid","name":"save","id":4,"data":[9]}), json!(true)),
                (json!({"op":"recordNextId","pid":"pid","name":"save"}), json!(5)),
                (json!({"op":"recordIds","pid":"pid","name":"save"}), json!([1, 4])),
                (json!({"op":"dbUsage","pid":"pid"}), json!(1)),
                (json!({"op":"recordDelete","pid":"pid","name":"save","id":1}), json!(true)),
            ] {
                let request: StorageRequest = serde_json::from_value(request).unwrap();
                assert_eq!(dispatch_storage(request, &store).await, expected);
            }
        });
        tauri::async_runtime::block_on(async {
            let store = store::open(root.path()).unwrap();
            for (request, expected) in [
                (json!({"op":"fileSize","aid":"aid","path":"save"}), json!(0)),
                (json!({"op":"recordGet","pid":"pid","name":"save","id":4}), json!([9])),
                (json!({"op":"dbDelete","pid":"pid","name":"save"}), json!(true)),
                (json!({"op":"recordGet","pid":"pid","name":"save","id":4}), Value::Null),
                (json!({"op":"dbExists","pid":"pid","name":"save"}), json!(false)),
            ] {
                let request: StorageRequest = serde_json::from_value(request).unwrap();
                assert_eq!(dispatch_storage(request, &store).await, expected);
            }
        });
    }

    #[test]
    fn audio_wire_preserves_native_sequence_and_rejects_invalid_input() {
        let request: AudioRequest = serde_json::from_value(json!({
            "type":"play", "handle":3, "duration":50, "repeat":true,
            "events":[
                {"time":0,"kind":"midi","data":[144,60,100]},
                {"time":10,"kind":"wave","channels":2,"samplingRate":8000,"samples":[-32768,32767]}
            ]
        }))
        .unwrap();
        let AudioCommand::Play { handle, sequence, repeat } = request.try_into().unwrap() else {
            panic!("Expected play");
        };
        assert_eq!(handle, 3);
        assert!(repeat);
        assert_eq!(sequence.duration, 50);
        assert_eq!(sequence.events[0].data, AudioEventData::Midi(vec![144, 60, 100]));
        assert_eq!(sequence.events[1].time, 10);
        assert_eq!(
            sequence.events[1].data,
            AudioEventData::Wave {
                channels: 2,
                sampling_rate: 8000,
                samples: vec![-32768, 32767]
            }
        );
        for (channels, sampling_rate, samples) in [(0, 8000, vec![0]), (2, 0, vec![0, 0]), (2, 8000, vec![0])] {
            let request: AudioRequest = serde_json::from_value(json!({
                "type":"play","handle":1,"duration":1,"repeat":false,
                "events":[{"time":0,"kind":"wave","channels":channels,"samplingRate":sampling_rate,"samples":samples}]
            }))
            .unwrap();
            assert!(AudioCommand::try_from(request).is_err());
        }
        for events in [
            json!([{"time":0,"kind":"midi","data":[144]}]),
            json!([{"time":2,"kind":"midi","data":[144,60,100]}]),
            json!([{"time":1,"kind":"midi","data":[144,60,100]}, {"time":0,"kind":"midi","data":[128,60,0]}]),
        ] {
            let request: AudioRequest = serde_json::from_value(json!({
                "type":"play","handle":1,"duration":1,"repeat":false,"events":events
            }))
            .unwrap();
            assert!(AudioCommand::try_from(request).is_err());
        }
        let request: AudioRequest = serde_json::from_value(json!({"type":"stop","handle":3})).unwrap();
        assert!(matches!(AudioCommand::try_from(request).unwrap(), AudioCommand::Stop { handle: 3 }));
        assert!(
            serde_json::from_value::<StorageRequest>(json!({
                "op":"fileRead","aid":"aid","path":"save","offset":-1,"count":8
            }))
            .is_err()
        );
    }
}
