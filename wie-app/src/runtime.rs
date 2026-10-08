use std::{
    collections::{HashMap, hash_map::Entry},
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, RecvTimeoutError, Sender},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::{Result, anyhow};
use futures::{
    FutureExt,
    channel::oneshot,
    future::{BoxFuture, Shared},
    lock::Mutex as AsyncMutex,
};
use serde::Serialize;
use tauri::{AppHandle, Manager, State, ipc::Channel};

use wie::load_emulator;
use wie_backend::{
    AudioSink, DatabaseRepository as BackendDatabaseRepository, Event, Filesystem, Font, Instant as GuestInstant, KeyCode, Options, Platform, Screen,
};
#[cfg(not(target_os = "ios"))]
use wie_core_arm_native::NativeExecutor;

use crate::{
    audio::{Audio, AudioSink as NativeAudioSink},
    database::DatabaseRepository,
    filesystem::SqliteFilesystem,
    library::{Library, LibraryApp},
    screen::{NativeScreen, NativeView},
    settings::{Settings, SettingsStore},
    store::{self, Store},
};

#[cfg(target_os = "ios")]
pub(crate) mod ios;

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SessionEvent {
    Ready,
    Warning { message: String },
    Stopped,
    Error { message: String },
}

type Completion = Shared<BoxFuture<'static, Result<(), String>>>;

struct Session {
    id: u64,
    app_id: String,
    commands: Sender<Command>,
    completion: Completion,
}

enum Command {
    Key(KeyCode, bool),
    ReleaseKeys,
    Volumes(f32, f32),
    #[cfg(mobile)]
    Suspend(bool),
    Stop,
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
    view: NativeView,
    runtime: Mutex<Runtime>,
}

impl AppState {
    pub fn new(app: AppHandle, root: PathBuf, view: NativeView) -> Result<Self> {
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
            view,
        })
    }

    pub fn stop(&self) -> Option<Completion> {
        let runtime = self.runtime.lock().unwrap();
        let session = runtime.session.as_ref()?;
        let _ = session.commands.send(Command::Stop);
        Some(session.completion.clone())
    }

    pub fn release_keys(&self) {
        if let Some(session) = &self.runtime.lock().unwrap().session {
            let _ = session.commands.send(Command::ReleaseKeys);
        }
    }

    #[cfg(mobile)]
    pub fn suspend(&self, paused: bool) {
        let mut runtime = self.runtime.lock().unwrap();
        runtime.suspended = paused;
        if let Some(session) = &runtime.session {
            let _ = session.commands.send(Command::Suspend(paused));
        }
    }
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
    if let Some(session) = &state.runtime.lock().unwrap().session {
        let _ = session.commands.send(volumes);
    }
    Ok(())
}

#[tauri::command]
pub async fn start_game(app: AppHandle, state: State<'_, AppState>, id: String, events: Channel<SessionEvent>) -> Result<u64, String> {
    let (session_id, startup, completion) = {
        let settings_store = state.settings.lock().await;
        let settings = settings_store.read().await.map_err(|error| error.to_string())?;
        tauri::async_runtime::spawn_blocking(move || {
            let state = app.state::<AppState>();
            // Library operations serialize deletion with session registration without blocking input.
            let library = state.library.lock().unwrap();
            if state.runtime.lock().unwrap().session.is_some() {
                return Err("A game is already running".to_owned());
            }
            let (metadata, bytes) = library.read_archive(&id).map_err(|error| error.to_string())?;
            let mut runtime = state.runtime.lock().unwrap();
            let session_id = runtime.next_id;
            runtime.next_id += 1;
            let (sender, receiver) = mpsc::channel();
            let (started, startup) = oneshot::channel();
            let initialized = Arc::new(AtomicBool::new(false));
            let ready = initialized.clone();
            let store = state.store.clone();
            let view = state.view.clone();
            let worker_app = app.clone();
            let worker_events = events.clone();
            #[cfg(mobile)]
            let suspended = runtime.suspended;
            let worker = tauri::async_runtime::spawn_blocking(move || {
                SessionWorker {
                    app: worker_app,
                    store,
                    view,
                    filename: metadata.filename,
                    bytes,
                    settings,
                    events: worker_events,
                    initialized: ready,
                    #[cfg(mobile)]
                    suspended,
                }
                .run(receiver, started)
                .map_err(|error| format!("{error:#}"))
            });
            let view = state.view.clone();
            let app = app.clone();
            let completion = async move {
                let result = match worker.await {
                    Ok(result) => result,
                    Err(error) => Err(format!("Game worker failed: {error}")),
                };
                let cleanup = view.set_playing(false).map_err(|error| error.to_string());
                let result = result.and(cleanup);
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
                commands: sender,
                completion: completion.clone(),
            });
            Ok((session_id, startup, completion))
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
        return Err("Game stopped before initialization completed".into());
    }
    Ok(session_id)
}

#[tauri::command]
pub async fn key_event(state: State<'_, AppState>, session_id: u64, key: String, pressed: bool) -> Result<(), String> {
    let key = match key.as_str() {
        "UP" => KeyCode::UP,
        "DOWN" => KeyCode::DOWN,
        "LEFT" => KeyCode::LEFT,
        "RIGHT" => KeyCode::RIGHT,
        "OK" => KeyCode::OK,
        "0" => KeyCode::NUM0,
        "1" => KeyCode::NUM1,
        "2" => KeyCode::NUM2,
        "3" => KeyCode::NUM3,
        "4" => KeyCode::NUM4,
        "5" => KeyCode::NUM5,
        "6" => KeyCode::NUM6,
        "7" => KeyCode::NUM7,
        "8" => KeyCode::NUM8,
        "9" => KeyCode::NUM9,
        "#" => KeyCode::HASH,
        "*" => KeyCode::STAR,
        "CLR" => KeyCode::CLEAR,
        _ => return Err(format!("Unknown key: {key}")),
    };
    let runtime = state.runtime.lock().unwrap();
    if let Some(session) = &runtime.session
        && session.id == session_id
    {
        let _ = session.commands.send(Command::Key(key, pressed));
    }
    Ok(())
}

#[tauri::command]
pub async fn release_keys(state: State<'_, AppState>, session_id: u64) -> Result<(), String> {
    let runtime = state.runtime.lock().unwrap();
    if let Some(session) = &runtime.session
        && session.id == session_id
    {
        let _ = session.commands.send(Command::ReleaseKeys);
    }
    Ok(())
}

#[tauri::command]
pub async fn stop_game(state: State<'_, AppState>, session_id: u64) -> Result<(), String> {
    let completion = {
        let runtime = state.runtime.lock().unwrap();
        runtime.session.as_ref().filter(|session| session.id == session_id).map(|session| {
            let _ = session.commands.send(Command::Stop);
            session.completion.clone()
        })
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

struct NativePlatform {
    app: AppHandle,
    screen: NativeScreen,
    audio: NativeAudioSink,
    database: DatabaseRepository,
    filesystem: SqliteFilesystem,
    clock: Arc<Mutex<Clock>>,
    exited: Arc<AtomicBool>,
    font: Font,
}

impl Platform for NativePlatform {
    fn font(&self) -> &Font {
        &self.font
    }

    fn screen(&self) -> &dyn Screen {
        &self.screen
    }

    fn now(&self) -> GuestInstant {
        self.clock.lock().unwrap().now()
    }

    fn database_repository(&self) -> &dyn BackendDatabaseRepository {
        &self.database
    }

    fn filesystem(&self) -> &dyn Filesystem {
        &self.filesystem
    }

    fn audio_sink(&self) -> Box<dyn AudioSink> {
        Box::new(self.audio.clone())
    }

    fn write_stdout(&self, bytes: &[u8]) {
        log::info!("{}", String::from_utf8_lossy(bytes));
    }

    fn write_stderr(&self, bytes: &[u8]) {
        log::warn!("{}", String::from_utf8_lossy(bytes));
    }

    fn exit(&self) {
        self.exited.store(true, Ordering::Release);
    }

    fn vibrate(&self, duration_ms: u64, intensity: u8) {
        #[cfg(mobile)]
        {
            use tauri_plugin_haptics::HapticsExt;
            if duration_ms != 0
                && intensity != 0
                && let Err(error) = self.app.haptics().vibrate(duration_ms.min(u64::from(u32::MAX)) as u32)
            {
                log::warn!("Vibration failed: {error}");
            }
        }
        #[cfg(desktop)]
        let _ = (&self.app, duration_ms, intensity);
    }
}

struct SessionWorker {
    app: AppHandle,
    store: Store,
    view: NativeView,
    filename: String,
    bytes: Vec<u8>,
    settings: Settings,
    events: Channel<SessionEvent>,
    initialized: Arc<AtomicBool>,
    #[cfg(mobile)]
    suspended: bool,
}

impl SessionWorker {
    fn run(self, commands: Receiver<Command>, started: oneshot::Sender<()>) -> Result<()> {
        let Self {
            app,
            store,
            view,
            filename,
            bytes,
            settings,
            events,
            initialized,
            #[cfg(mobile)]
            suspended,
        } = self;
        let warnings = events.clone();
        let mut audio = Audio::new(&app, settings.midi_volume, settings.pcm_volume, move |message| {
            let _ = warnings.send(SessionEvent::Warning { message });
        })?;
        let clock = Arc::new(Mutex::new(Clock::new()?));
        #[cfg(mobile)]
        {
            clock.lock().unwrap().set_paused(suspended);
            audio.pause(suspended)?;
        }
        let redraw = Arc::new(AtomicBool::new(true));
        let exited = Arc::new(AtomicBool::new(false));
        let platform = NativePlatform {
            app,
            screen: NativeScreen::new(view.clone(), 240, 320, redraw.clone()),
            audio: audio.sink(),
            database: DatabaseRepository { store: store.clone() },
            filesystem: SqliteFilesystem { store },
            clock: clock.clone(),
            exited: exited.clone(),
            font: Font::try_from_static(include_bytes!("../../assets/neodgm.ttf"))?,
        };
        let result = (|| -> Result<()> {
            view.set_playing(true)?;
            let mut emulator = load_emulator(
                &filename,
                bytes,
                Box::new(platform),
                Options {
                    enable_gdbserver: false,
                    #[cfg(not(target_os = "ios"))]
                    aot: Some(Box::new(NativeExecutor::new())),
                    #[cfg(target_os = "ios")]
                    aot: None,
                    profile: None,
                },
            )?;
            initialized.store(true, Ordering::Release);
            let _ = started.send(());
            let mut keys = HashMap::new();
            let mut ready = false;
            #[cfg(mobile)]
            let mut paused = suspended;
            #[cfg(desktop)]
            let paused = false;
            loop {
                let command = match commands.recv_timeout(Duration::from_millis(if paused { 100 } else { 1 })) {
                    Ok(command) => Some(command),
                    Err(RecvTimeoutError::Timeout) => None,
                    Err(RecvTimeoutError::Disconnected) => break,
                };
                if let Some(command) = command {
                    match command {
                        Command::Key(key, pressed) if ready && !paused => {
                            if pressed {
                                if let Entry::Vacant(entry) = keys.entry(key) {
                                    entry.insert(Instant::now());
                                    emulator.handle_event(Event::Keydown(key));
                                }
                            } else if keys.remove(&key).is_some() {
                                emulator.handle_event(Event::Keyup(key));
                            }
                        }
                        Command::Key(..) => {}
                        Command::ReleaseKeys => {
                            for (key, _) in keys.drain() {
                                emulator.handle_event(Event::Keyup(key));
                            }
                        }
                        Command::Volumes(midi, pcm) => audio.set_volumes(midi, pcm)?,
                        #[cfg(mobile)]
                        Command::Suspend(suspended) => {
                            for (key, _) in keys.drain() {
                                emulator.handle_event(Event::Keyup(key));
                            }
                            clock.lock().unwrap().set_paused(suspended);
                            audio.pause(suspended)?;
                            paused = suspended;
                        }
                        Command::Stop => break,
                    }
                }
                if exited.load(Ordering::Acquire) {
                    break;
                }
                if let Some(error) = view.take_error() {
                    return Err(anyhow!(error));
                }
                if paused {
                    continue;
                }
                if redraw.swap(false, Ordering::AcqRel) {
                    emulator.handle_event(Event::Redraw);
                }
                for (&key, last) in &mut keys {
                    if last.elapsed() >= Duration::from_millis(100) {
                        emulator.handle_event(Event::Keyrepeat(key));
                        *last = Instant::now();
                    }
                }
                emulator.tick()?;
                if !ready && !emulator.is_preparing() {
                    ready = true;
                    events.send(SessionEvent::Ready)?;
                }
            }
            for (key, _) in keys {
                emulator.handle_event(Event::Keyup(key));
            }
            Ok(())
        })();
        audio.shutdown();
        result
    }
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
