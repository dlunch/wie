use alloc::{
    boxed::Box,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use core::{
    mem, str,
    sync::atomic::{AtomicBool, Ordering},
};
use std::sync::Mutex;

use futures::{
    channel::oneshot,
    future::{AbortHandle, Abortable},
};
use hashbrown::HashMap;
use tracing_subscriber::{Layer, filter::LevelFilter, fmt::time::UtcTime, layer::SubscriberExt, util::SubscriberInitExt};
use tracing_web::MakeConsoleWriter;
use wasm_bindgen::{JsError, prelude::*};
use wasm_bindgen_futures::spawn_local;
use web_sys::{AbortSignal, HtmlCanvasElement};
use wgpu::{FilterMode, Instance, InstanceDescriptor, SurfaceTarget};

use wie_backend::{DatabaseRepository as BackendDatabaseRepository, Emulator, Event, Filesystem, Font, Instant, KeyCode, Options, Platform, Screen};
use wie_renderer::{GpuContext, Presentation, Presenter, RenderTarget};

use crate::{
    aot,
    audio_sink::{AudioPlayer, AudioSink},
    database::DatabaseRepository,
    filesystem::WebFilesystem,
    host::WasmHost,
    window::{Pending, WindowImpl},
};

#[wasm_bindgen(module = "panic.ts")]
extern "C" {
    #[wasm_bindgen(js_name = recordPanic)]
    fn record_panic(message: &str);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "import('../src/ts/backend/wasm').PresentationOptions")]
    pub type PresentationOptions;

    #[wasm_bindgen(method, getter, js_name = scaleFilter)]
    fn scale_filter(this: &PresentationOptions) -> String;

    #[wasm_bindgen(method, getter, js_name = onError)]
    fn on_error(this: &PresentationOptions) -> js_sys::Function;

    #[wasm_bindgen(method, getter)]
    fn signal(this: &PresentationOptions) -> AbortSignal;
}

struct WieWebPlatform {
    audio_player: AudioPlayer,
    database_repository: Box<dyn BackendDatabaseRepository>,
    filesystem: Box<dyn Filesystem>,
    host: Option<WasmHost>,
    exited: Arc<AtomicBool>,
    font: Font,
    window: WindowImpl,
}

// XXX we're on single thread
unsafe impl Sync for WieWebPlatform {}
unsafe impl Send for WieWebPlatform {}

impl WieWebPlatform {
    fn new(window: WindowImpl, font: Font, audio_player: AudioPlayer, host: Option<WasmHost>, exited: Arc<AtomicBool>) -> Self {
        let (database_repository, filesystem): (Box<dyn BackendDatabaseRepository>, Box<dyn Filesystem>) = match &host {
            Some(host) => (Box::new(host.clone()), Box::new(host.clone())),
            None => (Box::new(DatabaseRepository::new()), Box::new(WebFilesystem::new())),
        };
        Self {
            audio_player,
            database_repository,
            filesystem,
            host,
            exited,
            font,
            window,
        }
    }
}

impl Platform for WieWebPlatform {
    fn font(&self) -> &Font {
        &self.font
    }

    fn screen(&self) -> &dyn Screen {
        &self.window
    }

    fn now(&self) -> Instant {
        let millis = self.host.as_ref().map_or_else(js_sys::Date::now, WasmHost::now);

        Instant::from_epoch_millis(millis as _)
    }

    fn database_repository(&self) -> &dyn BackendDatabaseRepository {
        self.database_repository.as_ref()
    }

    fn filesystem(&self) -> &dyn Filesystem {
        self.filesystem.as_ref()
    }

    fn audio_sink(&self) -> Box<dyn wie_backend::AudioSink> {
        Box::new(AudioSink::new(self.audio_player.clone()))
    }

    fn write_stdout(&self, data: &[u8]) {
        let string = str::from_utf8(data).unwrap();
        tracing::info!("{}", string);
    }

    fn write_stderr(&self, data: &[u8]) {
        let string = str::from_utf8(data).unwrap();
        tracing::info!("{}", string);
    }

    fn exit(&self) {
        self.exited.store(true, Ordering::SeqCst);
    }

    fn vibrate(&self, duration_ms: u64, intensity: u8) {
        if let Some(host) = &self.host {
            host.vibrate(duration_ms as f64, intensity);
            return;
        }

        if duration_ms == 0 || intensity == 0 {
            return;
        }

        let Some(window) = web_sys::window() else { return };
        let navigator = window.navigator();
        if !js_sys::Reflect::has(navigator.as_ref(), &JsValue::from_str("vibrate")).unwrap_or(false) {
            return;
        }
        let duration = core::cmp::min(duration_ms, u32::MAX as u64) as u32;
        navigator.vibrate_with_duration(duration);
    }
}

#[wasm_bindgen]
pub struct WieWeb {
    emulator: Box<dyn Emulator>,
    audio_player: AudioPlayer,
    host: Option<WasmHost>,
    exited: Arc<AtomicBool>,
    pending: Arc<Mutex<Pending>>,
    key_events: HashMap<KeyCode, f64>,
    notification: AbortHandle,
    presenter: Option<Presenter<'static>>,
    target: Option<RenderTarget>,
    context: GpuContext,
    instance: Instance,
    canvas: HtmlCanvasElement,
    filter: FilterMode,
    retry_used: bool,
    waiting_for_output: bool,
}

impl Drop for WieWeb {
    fn drop(&mut self) {
        self.notification.abort();
        // Runtime tasks can retain platform references after the view closes.
        self.audio_player.dispose();
    }
}

#[wasm_bindgen]
pub struct ImportedAppMetadata {
    id: String,
    title: String,
    icon: Vec<u8>,
}

#[wasm_bindgen]
impl ImportedAppMetadata {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn title(&self) -> String {
        self.title.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn icon(&self) -> Vec<u8> {
        self.icon.clone()
    }
}

#[wasm_bindgen(js_name = extractAppMetadata)]
pub fn extract_app_metadata(filename: &str, buf: &[u8]) -> Result<ImportedAppMetadata, JsError> {
    let file_type = wie::determine_file_type(filename, buf).map_err(|error| JsError::new(&error.to_string()))?;
    let metadata = wie::extract_app_metadata(filename, buf, file_type).map_err(|error| JsError::new(&error.to_string()))?;
    Ok(ImportedAppMetadata {
        id: metadata.id,
        title: metadata.title,
        icon: metadata.icon.unwrap_or_default(),
    })
}

#[wasm_bindgen]
impl WieWeb {
    pub async fn create(
        filename: &str,
        buf: &[u8],
        canvas: HtmlCanvasElement,
        font_data: Vec<u8>,
        audio_player: AudioPlayer,
        host: Option<WasmHost>,
        presentation: PresentationOptions,
    ) -> Result<WieWeb, JsError> {
        let filter = match presentation.scale_filter().as_str() {
            "linear" => FilterMode::Linear,
            "nearest" => FilterMode::Nearest,
            _ => return Err(JsError::new("Unsupported scale filter")),
        };
        let on_error = presentation.on_error();
        let signal = presentation.signal();
        let (sender, receiver) = oneshot::channel::<anyhow::Error>();
        let sender = Mutex::new(Some(sender));
        let (notification, registration) = AbortHandle::new_pair();
        let notification_signal = signal.clone();
        spawn_local(async move {
            if let Ok(Ok(error)) = Abortable::new(receiver, registration).await
                && !notification_signal.aborted()
            {
                let _ = on_error.call1(&JsValue::NULL, &JsError::new(&error.to_string()).into());
            }
        });
        let abort_notification = notification.clone();
        let on_abort = Closure::<dyn FnMut()>::new(move || abort_notification.abort());
        signal
            .add_event_listener_with_callback("abort", on_abort.as_ref().unchecked_ref())
            .map_err(|error| JsError::new(&alloc::format!("{error:?}")))?;
        if signal.aborted() {
            notification.abort();
        }
        let result = async {
            let instance = wgpu::util::new_instance_with_webgpu_detection(InstanceDescriptor::new_without_display_handle()).await;
            anyhow::ensure!(!signal.aborted(), "Presentation startup canceled");
            let surface = instance.create_surface(SurfaceTarget::Canvas(canvas.clone()))?;
            let context = GpuContext::new(&instance, &surface, move |error| {
                if let Some(sender) = sender.lock().unwrap().take() {
                    let _ = sender.send(error);
                }
            })
            .await?;
            anyhow::ensure!(!signal.aborted(), "Presentation startup canceled");
            let presenter = Presenter::new(&context, surface, filter)?;
            let pending = Arc::new(Mutex::new(Pending {
                width: 240,
                height: 320,
                redraw: true,
                frame: None,
            }));
            let exited = Arc::new(AtomicBool::new(false));
            let window = WindowImpl { pending: pending.clone() };
            let font = Font::try_from_vec(font_data)?;
            let platform = Box::new(WieWebPlatform::new(window, font, audio_player.clone(), host.clone(), exited.clone()));
            let options = Options {
                enable_gdbserver: false,
                aot: Some(Box::new(aot::WasmExecutor::default())),
                profile: None,
            };

            let emulator = wie::load_emulator(filename, buf.to_vec(), platform, options)?;

            anyhow::Ok(Self {
                emulator,
                audio_player,
                host,
                exited,
                pending,
                key_events: HashMap::new(),
                notification: notification.clone(),
                presenter: Some(presenter),
                target: None,
                context,
                instance,
                canvas,
                filter,
                retry_used: false,
                waiting_for_output: true,
            })
        }
        .await;
        let _ = signal.remove_event_listener_with_callback("abort", on_abort.as_ref().unchecked_ref());
        if result.is_err() {
            notification.abort();
        }
        result.map_err(|error| JsError::new(&error.to_string()))
    }

    pub fn is_preparing(&self) -> bool {
        self.emulator.is_preparing()
    }

    pub fn is_exited(&self) -> bool {
        self.exited.load(Ordering::SeqCst)
    }

    pub fn update(&mut self) -> Result<bool, JsError> {
        let redraw = mem::take(&mut self.pending.lock().unwrap().redraw);
        if redraw {
            self.emulator.handle_event(Event::Redraw);
        }

        let millis = self.host.as_ref().map_or_else(js_sys::Date::now, WasmHost::now);

        for (key, key_millis) in self.key_events.iter_mut() {
            if millis - *key_millis > 100.0 {
                self.emulator.handle_event(Event::Keyrepeat(*key));
                *key_millis = millis;
            }
        }

        self.emulator.tick().map_err(|e| JsError::new(&e.to_string()))?;
        if !self.waiting_for_output && self.upload_pending().map_err(|error| JsError::new(&error.to_string()))? {
            return self.present().map_err(|error| JsError::new(&error.to_string()));
        }
        Ok(self.retry_used && !self.waiting_for_output)
    }

    /// Redraw retained content without advancing the guest, including while suspended.
    pub fn redraw(&mut self, width: u32, height: u32, output_event: bool) -> Result<bool, JsError> {
        (|| {
            if output_event && width > 0 && height > 0 && self.waiting_for_output {
                self.waiting_for_output = false;
                self.retry_used = false;
            }
            if self.canvas.width() != width || self.canvas.height() != height || output_event {
                self.canvas.set_width(width);
                self.canvas.set_height(height);
                self.presenter.as_mut().unwrap().resize(&self.context, width, height)?;
            }
            if width == 0 || height == 0 {
                self.waiting_for_output = true;
            }
            if !self.waiting_for_output {
                self.upload_pending()?;
            }
            self.present()
        })()
        .map_err(|error| JsError::new(&error.to_string()))
    }

    pub fn key_down(&mut self, key: String) -> Result<(), JsError> {
        let millis = self.host.as_ref().map_or_else(js_sys::Date::now, WasmHost::now);
        let key = KeyCode::parse(&key);

        self.emulator.handle_event(Event::Keydown(key));
        self.key_events.insert(key, millis);

        Ok(())
    }

    pub fn key_up(&mut self, key: String) -> Result<(), JsError> {
        let key = KeyCode::parse(&key);

        self.emulator.handle_event(Event::Keyup(key));
        self.key_events.remove(&key);

        Ok(())
    }
}

impl WieWeb {
    fn upload_pending(&mut self) -> anyhow::Result<bool> {
        let frame = self.pending.lock().unwrap().frame.take();
        let Some(frame) = frame else { return Ok(false) };
        if self
            .target
            .as_ref()
            .is_none_or(|target| target.width != frame.width || target.height != frame.height)
        {
            self.target = Some(RenderTarget::new(&self.context, frame.width, frame.height)?);
        }
        self.target.as_ref().unwrap().upload(&self.context, &frame);
        Ok(true)
    }

    fn present(&mut self) -> anyhow::Result<bool> {
        if self.waiting_for_output {
            return Ok(false);
        }
        match self.presenter.as_mut().unwrap().present(&self.context, self.target.as_ref())? {
            Presentation::Presented => self.retry_used = false,
            Presentation::WaitForOutput => self.waiting_for_output = true,
            status @ (Presentation::Retry | Presentation::RecreateSurface) => {
                anyhow::ensure!(!self.retry_used, "Presentation failed after one retry");
                self.retry_used = true;
                if matches!(status, Presentation::RecreateSurface) {
                    self.presenter = None;
                    let surface = self.instance.create_surface(SurfaceTarget::Canvas(self.canvas.clone()))?;
                    let mut presenter = Presenter::new(&self.context, surface, self.filter)?;
                    presenter.resize(&self.context, self.canvas.width(), self.canvas.height())?;
                    self.presenter = Some(presenter);
                }
                return Ok(true);
            }
        }
        Ok(false)
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    use std::panic;

    panic::set_hook(Box::new(|info| record_panic(&info.to_string())));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_timer(UtcTime::rfc_3339())
        .with_writer(MakeConsoleWriter)
        .with_filter(LevelFilter::INFO);

    tracing_subscriber::registry().with(fmt_layer).init();
}
