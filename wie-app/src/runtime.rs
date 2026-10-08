use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Sender},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use futures::{
    FutureExt,
    channel::oneshot,
    future::{BoxFuture, Shared},
    lock::Mutex as AsyncMutex,
};
use serde::Serialize;
use tauri::{AppHandle, Manager, State, ipc::Channel};

use wie_backend::Instant as GuestInstant;

use crate::{
    library::{Library, LibraryApp},
    settings::{Settings, SettingsStore},
    store::{self, Store},
};

#[cfg(not(target_os = "ios"))]
use crate::screen::NativeView;

use platform::{Command, StartedApp};

#[cfg_attr(target_os = "ios", path = "runtime/ios.rs")]
#[cfg_attr(not(target_os = "ios"), path = "runtime/native.rs")]
pub(crate) mod platform;

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SessionEvent {
    #[cfg(not(target_os = "ios"))]
    Ready,
    #[cfg(target_os = "ios")]
    Lifecycle {
        suspended: bool,
        #[serde(rename = "guestTimeMs")]
        guest_time_ms: u64,
    },
    Warning {
        message: String,
    },
    Stopped,
    Error {
        message: String,
    },
}

type Completion = Shared<BoxFuture<'static, Result<(), String>>>;

struct Session {
    id: u64,
    app_id: String,
    commands: Option<Sender<Command>>,
    completion: Completion,
}

impl Session {
    fn stop(&mut self) -> Completion {
        // Taking the sender closes admission before the ordered stop is enqueued.
        if let Some(commands) = self.commands.take() {
            let _ = commands.send(Command::Stop);
        }
        self.completion.clone()
    }
}

struct Runtime {
    session: Option<Session>,
    next_id: u64,
    #[cfg(mobile)]
    suspended: bool,
}

pub struct AppState {
    store: Store,
    library: Arc<Mutex<Library>>,
    settings: AsyncMutex<SettingsStore>,
    #[cfg(not(target_os = "ios"))]
    view: NativeView,
    runtime: Mutex<Runtime>,
}

impl AppState {
    pub fn new(app: AppHandle, root: PathBuf, #[cfg(not(target_os = "ios"))] view: NativeView) -> Result<Self> {
        let store = store::open(&root)?;
        Ok(Self {
            runtime: Mutex::new(Runtime {
                session: None,
                next_id: 1,
                #[cfg(mobile)]
                suspended: false,
            }),
            library: Arc::new(Mutex::new(Library::new(root)?)),
            settings: AsyncMutex::new(SettingsStore::new(app)),
            store,
            #[cfg(not(target_os = "ios"))]
            view,
        })
    }

    pub fn stop_app(&self) -> Option<Completion> {
        self.runtime.lock().unwrap().session.as_mut().map(Session::stop)
    }

    #[cfg(mobile)]
    pub fn suspend(&self, paused: bool) {
        let mut runtime = self.runtime.lock().unwrap();
        runtime.suspended = paused;
        if let Some(session) = &runtime.session
            && let Some(commands) = &session.commands
        {
            let _ = commands.send(Command::Suspend(paused));
        }
    }
}

#[tauri::command]
pub fn runtime_kind() -> &'static str {
    if cfg!(target_os = "ios") { "wasm" } else { "native" }
}

#[tauri::command]
pub async fn list_apps(state: State<'_, AppState>) -> Result<Vec<LibraryApp>, String> {
    let library = state.library.clone();
    tauri::async_runtime::spawn_blocking(move || library.lock().unwrap().list())
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn import_app(state: State<'_, AppState>, filename: String, bytes: Vec<u8>) -> Result<(), String> {
    let library = state.library.clone();
    tauri::async_runtime::spawn_blocking(move || library.lock().unwrap().import(&filename, &bytes))
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn delete_app(app: AppHandle, id: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let mut library = state.library.lock().unwrap();
        if state.runtime.lock().unwrap().session.as_ref().is_some_and(|session| session.app_id == id) {
            return Err("Cannot delete a running app".into());
        }
        library.delete(&id).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn read_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    state.settings.lock().await.read().await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn write_settings(state: State<'_, AppState>, settings: Settings) -> Result<(), String> {
    let store = state.settings.lock().await;
    let volumes = Command::Volumes(settings.midi_volume, settings.pcm_volume);
    store.write(settings).await.map_err(|error| error.to_string())?;
    if let Some(session) = &state.runtime.lock().unwrap().session
        && let Some(commands) = &session.commands
    {
        let _ = commands.send(volumes);
    }
    Ok(())
}

#[tauri::command]
pub async fn start_app(app: AppHandle, state: State<'_, AppState>, id: String, events: Channel<SessionEvent>) -> Result<StartedApp, String> {
    let (started_app, startup, completion) = {
        let settings_store = state.settings.lock().await;
        let settings = settings_store.read().await.map_err(|error| error.to_string())?;
        tauri::async_runtime::spawn_blocking(move || {
            let state = app.state::<AppState>();
            // Library operations serialize deletion with session registration without blocking input.
            let library = state.library.lock().unwrap();
            if state.runtime.lock().unwrap().session.is_some() {
                return Err("An app is already running".to_owned());
            }
            let (metadata, bytes) = library.read_archive(&id).map_err(|error| error.to_string())?;
            let mut runtime = state.runtime.lock().unwrap();
            let session_id = runtime.next_id;
            runtime.next_id += 1;
            let (sender, receiver) = mpsc::channel();
            let (started, startup) = oneshot::channel();
            let initialized = Arc::new(AtomicBool::new(false));
            let (started_app, worker) = SessionWorker {
                app: app.clone(),
                store: state.store.clone(),
                settings,
                events: events.clone(),
                initialized: initialized.clone(),
                #[cfg(mobile)]
                suspended: runtime.suspended,
            }
            .prepare(session_id, metadata.filename, bytes);
            let worker = tauri::async_runtime::spawn_blocking(move || worker.run(receiver, started).map_err(|error| format!("{error:#}")));
            #[cfg(not(target_os = "ios"))]
            let view = state.view.clone();
            let app = app.clone();
            let completion = async move {
                let result = match worker.await {
                    Ok(result) => result,
                    Err(error) => Err(format!("App worker failed: {error}")),
                };
                #[cfg(not(target_os = "ios"))]
                let result = result.and(view.set_playing(false).map_err(|error| error.to_string()));
                let state = app.state::<AppState>();
                state.runtime.lock().unwrap().session = None;
                if initialized.load(Ordering::Acquire) {
                    let event = match &result {
                        Ok(()) => SessionEvent::Stopped,
                        Err(message) => SessionEvent::Error { message: message.clone() },
                    };
                    let _ = events.send(event);
                }
                result
            }
            .boxed()
            .shared();
            runtime.session = Some(Session {
                id: session_id,
                app_id: id,
                commands: Some(sender),
                completion: completion.clone(),
            });
            Ok((started_app, startup, completion))
        })
        .await
        .map_err(|error| error.to_string())??
    };
    let watcher = completion.clone();
    tauri::async_runtime::spawn(async move {
        let _ = watcher.await;
    });
    if startup.await.is_err() {
        completion.await?;
        return Err("App stopped before initialization completed".into());
    }
    Ok(started_app)
}

#[tauri::command]
pub async fn stop_app(state: State<'_, AppState>, session_id: u64) -> Result<(), String> {
    let completion = {
        let mut runtime = state.runtime.lock().unwrap();
        runtime.session.as_mut().filter(|session| session.id == session_id).map(Session::stop)
    };
    if let Some(completion) = completion {
        completion.await?;
    }
    Ok(())
}

struct Clock {
    origin: Instant,
    epoch: u64,
    paused_at: Option<Instant>,
    paused_duration: Duration,
}

impl Clock {
    fn new() -> Result<Self> {
        Ok(Self {
            origin: Instant::now(),
            epoch: SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64,
            paused_at: None,
            paused_duration: Duration::ZERO,
        })
    }

    fn now(&self) -> GuestInstant {
        let elapsed = self.paused_at.unwrap_or_else(Instant::now).duration_since(self.origin) - self.paused_duration;
        GuestInstant::from_epoch_millis(self.epoch + elapsed.as_millis() as u64)
    }

    #[cfg(any(mobile, test))]
    fn set_paused(&mut self, paused: bool) {
        if paused {
            self.paused_at.get_or_insert_with(Instant::now);
        } else if let Some(since) = self.paused_at.take() {
            self.paused_duration += since.elapsed();
        }
    }
}

struct SessionWorker {
    app: AppHandle,
    store: Store,
    settings: Settings,
    events: Channel<SessionEvent>,
    initialized: Arc<AtomicBool>,
    #[cfg(mobile)]
    suspended: bool,
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::Clock;

    #[test]
    fn guest_clock_excludes_background_time() {
        let mut clock = Clock::new().unwrap();
        clock.set_paused(true);
        let before = clock.now().raw();
        clock.set_paused(true);
        assert_eq!(clock.now().raw(), before);
        let suspended = Duration::from_secs(20);
        clock.origin -= suspended;
        clock.paused_at = Some(Instant::now() - suspended);
        clock.set_paused(false);
        assert!(clock.now().raw() < clock.epoch + 1000);
        assert!(clock.paused_duration >= suspended);
        let duration = clock.paused_duration;
        clock.set_paused(false);
        assert_eq!(clock.paused_duration, duration);
    }
}
