#![no_std]
extern crate alloc;

mod aot;
mod audio_sink;
mod database;
mod filesystem;
mod indexed_db_store;
mod util;
mod window;

use alloc::{
    boxed::Box,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use core::{
    str,
    sync::atomic::{AtomicBool, Ordering},
};

use hashbrown::HashMap;
use tracing_subscriber::{Layer, filter::LevelFilter, fmt::time::UtcTime, layer::SubscriberExt, util::SubscriberInitExt};
use tracing_web::MakeConsoleWriter;
use wasm_bindgen::{JsError, prelude::*};
use web_sys::HtmlCanvasElement;

use wie_backend::{Emulator, Event, Font, Instant, KeyCode, Options, Platform, Screen};

use self::{
    audio_sink::{AudioPlayer, AudioSink},
    database::DatabaseRepository,
    filesystem::WebFilesystem,
    window::WindowImpl,
};

struct WieWebPlatform {
    audio_player: AudioPlayer,
    database_repository: DatabaseRepository,
    filesystem: WebFilesystem,
    font: Font,
    window: WindowImpl,
}

// XXX we're on single thread
unsafe impl Sync for WieWebPlatform {}
unsafe impl Send for WieWebPlatform {}

impl WieWebPlatform {
    fn new(window: WindowImpl, font: Font, audio_player: AudioPlayer) -> Self {
        Self {
            audio_player,
            database_repository: DatabaseRepository::new(),
            filesystem: WebFilesystem::new(),
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
        let date = js_sys::Date::new_0();
        let millis = date.value_of();

        Instant::from_epoch_millis(millis as _)
    }

    fn database_repository(&self) -> &dyn wie_backend::DatabaseRepository {
        &self.database_repository
    }

    fn filesystem(&self) -> &dyn wie_backend::Filesystem {
        &self.filesystem
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

    fn exit(&self) {}

    fn vibrate(&self, duration_ms: u64, intensity: u8) {
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
    should_redraw: Arc<AtomicBool>,
    key_events: HashMap<KeyCode, f64>,
}

impl Drop for WieWeb {
    fn drop(&mut self) {
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
    let metadata = wie::extract_app_metadata(filename, buf).map_err(|error| JsError::new(&error.to_string()))?;
    Ok(ImportedAppMetadata {
        id: metadata.id,
        title: metadata.title,
        icon: metadata.icon.unwrap_or_default(),
    })
}

#[wasm_bindgen]
impl WieWeb {
    #[wasm_bindgen(constructor)]
    pub fn new(filename: &str, buf: &[u8], canvas: HtmlCanvasElement, font_data: Vec<u8>, enable_aot: bool) -> Result<WieWeb, JsError> {
        let audio_player = AudioPlayer::new();
        let result = (|| {
            let should_redraw = Arc::new(AtomicBool::new(true));
            let window = WindowImpl::new(canvas, should_redraw.clone());
            let font = Font::try_from_vec(font_data)?;
            let platform = Box::new(WieWebPlatform::new(window, font, audio_player.clone()));
            let options = Options {
                enable_gdbserver: false,
                aot: enable_aot.then(|| Box::new(aot::WasmExecutor::default()) as Box<dyn wie_arm_jit_types::CompiledExecutor>),
                profile: None,
            };

            let emulator = wie::load_emulator(filename, buf.to_vec(), platform, options)?;

            anyhow::Ok(Self {
                emulator,
                audio_player: audio_player.clone(),
                should_redraw,
                key_events: HashMap::new(),
            })
        })();
        if result.is_err() {
            audio_player.dispose();
        }
        result.map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn is_preparing(&self) -> bool {
        self.emulator.is_preparing()
    }

    pub fn update(&mut self) -> Result<(), JsError> {
        if self.should_redraw.load(Ordering::SeqCst) {
            self.emulator.handle_event(Event::Redraw);
            self.should_redraw.store(false, Ordering::SeqCst)
        }

        let date = js_sys::Date::new_0();
        let millis = date.value_of();

        for (key, key_millis) in self.key_events.iter_mut() {
            if millis - *key_millis > 100.0 {
                self.emulator.handle_event(Event::Keyrepeat(*key));
                *key_millis = millis;
            }
        }

        self.emulator.tick().map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn key_down(&mut self, key: String) -> Result<(), JsError> {
        let date = js_sys::Date::new_0();
        let millis = date.value_of();
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

    pub fn set_pcm_volume(&self, volume: f32) {
        audio_sink::set_pcm_volume(volume);
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_timer(UtcTime::rfc_3339())
        .with_writer(MakeConsoleWriter)
        .with_filter(LevelFilter::INFO);

    tracing_subscriber::registry().with(fmt_layer).init();
}
