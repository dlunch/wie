use std::sync::Arc;
#[cfg(target_os = "ios")]
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::Receiver,
};

use anyhow::{Context, Error, Result, ensure};
#[cfg(target_os = "ios")]
use futures::channel::oneshot;
use midly::live::LiveEvent;
use serde::Deserialize;
#[cfg(target_os = "ios")]
use serde::Serialize;
use serde_json::{Value, json};
#[cfg(target_os = "ios")]
use tauri::{AppHandle, Manager, State, ipc::Channel};
#[cfg(target_os = "ios")]
use tauri_plugin_haptics::HapticsExt;
#[cfg(target_os = "ios")]
use wie_backend::AudioSink;
use wie_backend::{AudioCommand, AudioEventData, AudioSequence, Database, DatabaseRepository as _, Filesystem, TimedAudioEvent};

#[cfg(target_os = "ios")]
use crate::{audio::Audio, settings::Settings};
use crate::{
    database::{DatabaseRepository, SqliteDatabase},
    filesystem::SqliteFilesystem,
    store::Store,
};

#[cfg(target_os = "ios")]
use super::{AppState, Clock, Command, SessionEvent};

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub(crate) enum StorageRequest {
    FileExists {
        aid: String,
        path: String,
    },
    FileSize {
        aid: String,
        path: String,
    },
    FileRead {
        aid: String,
        path: String,
        offset: usize,
        count: usize,
    },
    FileWrite {
        aid: String,
        path: String,
        offset: usize,
        data: Vec<u8>,
    },
    FileTruncate {
        aid: String,
        path: String,
        length: usize,
    },
    DbOpen {
        pid: String,
        name: String,
    },
    DbExists {
        pid: String,
        name: String,
    },
    DbDelete {
        pid: String,
        name: String,
    },
    DbUsage {
        pid: String,
    },
    RecordNextId {
        pid: String,
        name: String,
    },
    RecordIds {
        pid: String,
        name: String,
    },
    RecordGet {
        pid: String,
        name: String,
        id: u32,
    },
    RecordDelete {
        pid: String,
        name: String,
        id: u32,
    },
    RecordAdd {
        pid: String,
        name: String,
        data: Vec<u8>,
    },
    RecordSet {
        pid: String,
        name: String,
        id: u32,
        data: Vec<u8>,
    },
}

impl StorageRequest {
    pub(crate) async fn dispatch(self, store: &Store) -> Value {
        let filesystem = SqliteFilesystem { store: store.clone() };
        let repository = DatabaseRepository { store: store.clone() };
        // Record operations use a proxy, not open(): reads must not create stores.
        let database = |pid, name| SqliteDatabase {
            store: store.clone(),
            pid,
            name,
        };
        match self {
            Self::FileExists { aid, path } => json!(filesystem.exists(&aid, &path).await),
            Self::FileSize { aid, path } => json!(filesystem.size(&aid, &path).await),
            Self::FileRead { aid, path, offset, count } => {
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
            Self::FileWrite { aid, path, offset, data } => {
                json!(filesystem.write(&aid, &path, offset, &data).await)
            }
            Self::FileTruncate { aid, path, length } => {
                filesystem.truncate(&aid, &path, length).await;
                Value::Null
            }
            Self::DbOpen { pid, name } => {
                repository.open(&name, &pid).await;
                Value::Null
            }
            Self::DbExists { pid, name } => json!(repository.exists(&name, &pid).await),
            Self::DbDelete { pid, name } => json!(repository.delete(&name, &pid).await),
            Self::DbUsage { pid } => json!(repository.usage(&pid).await),
            Self::RecordNextId { pid, name } => json!(database(pid, name).next_id().await),
            Self::RecordIds { pid, name } => json!(database(pid, name).get_record_ids().await),
            Self::RecordGet { pid, name, id } => json!(database(pid, name).get(id).await),
            Self::RecordDelete { pid, name, id } => json!(database(pid, name).delete(id).await),
            Self::RecordAdd { pid, name, data } => json!(database(pid, name).add(&data).await),
            Self::RecordSet { pid, name, id, data } => json!(database(pid, name).set(id, &data).await),
        }
    }
}

#[cfg(target_os = "ios")]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WebGame {
    pub session_id: u64,
    pub filename: String,
    pub bytes: Vec<u8>,
}

#[cfg(target_os = "ios")]
#[tauri::command]
pub(crate) async fn guest_storage(state: State<'_, AppState>, session_id: u64, request: StorageRequest) -> Result<Value, String> {
    let (reply, result) = oneshot::channel();
    state.runtime.lock().unwrap().send(session_id, Command::Storage(request, reply))?;
    result.await.map_err(|_| "Game worker stopped before completing storage".into())
}

#[cfg(target_os = "ios")]
#[tauri::command]
pub(crate) async fn guest_audio(state: State<'_, AppState>, session_id: u64, command: AudioRequest) -> Result<(), String> {
    let command = command.try_into().map_err(|error: Error| error.to_string())?;
    state.runtime.lock().unwrap().send(session_id, Command::Audio(command))
}

#[cfg(target_os = "ios")]
#[tauri::command]
pub(crate) async fn guest_vibrate(state: State<'_, AppState>, session_id: u64, duration_ms: u64, intensity: u8) -> Result<(), String> {
    state.runtime.lock().unwrap().send(session_id, Command::Vibrate(duration_ms, intensity))
}

#[cfg(target_os = "ios")]
pub(super) struct SessionWorker {
    pub app: AppHandle,
    pub store: Store,
    pub settings: Settings,
    pub events: Channel<SessionEvent>,
    pub initialized: Arc<AtomicBool>,
    pub suspended: bool,
}

#[cfg(target_os = "ios")]
impl SessionWorker {
    pub fn run(self, commands: Receiver<Command>, started: oneshot::Sender<()>) -> Result<()> {
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
                    let _ = reply.send(tauri::async_runtime::block_on(request.dispatch(&self.store)));
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
                let _ = self.app.state::<AppState>().stop();
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

    use super::{AudioRequest, StorageRequest};
    use crate::store;
    use wie_backend::{AudioCommand, AudioEventData};

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
                assert_eq!(request.dispatch(&store).await, expected);
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
                assert_eq!(request.dispatch(&store).await, expected);
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
