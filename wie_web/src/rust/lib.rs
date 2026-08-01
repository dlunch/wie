#![no_std]
extern crate alloc;

mod audio_sink;
mod database;
mod filesystem;
mod indexed_db_store;
mod util;
mod window;

use alloc::{
    borrow::ToOwned,
    boxed::Box,
    string::{String, ToString},
    sync::Arc,
};
use core::{
    str,
    sync::atomic::{AtomicBool, Ordering},
};

use hashbrown::HashMap;
use rodio::{DeviceSinkBuilder, Player};
use tracing_subscriber::{Layer, filter::LevelFilter, fmt::time::UtcTime, layer::SubscriberExt, util::SubscriberInitExt};
use tracing_web::MakeConsoleWriter;
use wasm_bindgen::{JsError, prelude::*};
use web_sys::HtmlCanvasElement;

use wie_backend::{Emulator, Event, Instant, KeyCode, Options, Platform, Screen, extract_zip};
use wie_j2me::J2MEEmulator;
use wie_ktf::KtfEmulator;
use wie_lgt::LgtEmulator;
use wie_skt::SktEmulator;

use self::{audio_sink::AudioSink, database::DatabaseRepository, filesystem::WebFilesystem, window::WindowImpl};

struct WieWebPlatform {
    database_repository: DatabaseRepository,
    filesystem: WebFilesystem,
    window: WindowImpl,
    player: Arc<Player>,
}

// XXX we're on single thread
unsafe impl Sync for WieWebPlatform {}
unsafe impl Send for WieWebPlatform {}

impl WieWebPlatform {
    fn new(window: WindowImpl, player: Arc<Player>) -> Self {
        Self {
            database_repository: DatabaseRepository::new(),
            filesystem: WebFilesystem::new(),
            window,
            player,
        }
    }
}

impl Platform for WieWebPlatform {
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
        Box::new(AudioSink::new(self.player.clone()))
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
    should_redraw: Arc<AtomicBool>,
    key_events: HashMap<KeyCode, f64>,
    player: Arc<Player>,
}

#[wasm_bindgen]
impl WieWeb {
    #[wasm_bindgen(constructor)]
    pub fn new(filename: &str, buf: &[u8], canvas: HtmlCanvasElement) -> Result<WieWeb, JsError> {
        (move || {
            let should_redraw = Arc::new(AtomicBool::new(true));
            let window = WindowImpl::new(canvas, should_redraw.clone());
            let output_stream = DeviceSinkBuilder::open_default_sink().unwrap();
            let player = Arc::new(Player::connect_new(output_stream.mixer()));
            let platform = Box::new(WieWebPlatform::new(window, player.clone()));
            let options = Options {
                enable_gdbserver: false,
                profile: None,
            };

            let emulator: Box<dyn Emulator> = if filename.ends_with("zip") {
                let files = extract_zip(buf).unwrap();

                if KtfEmulator::loadable_archive(&files) {
                    Box::new(KtfEmulator::from_archive(platform, files, options)?)
                } else if LgtEmulator::loadable_archive(&files) {
                    Box::new(LgtEmulator::from_archive(platform, files, options)?)
                } else if SktEmulator::loadable_archive(&files) {
                    Box::new(SktEmulator::from_archive(platform, files)?)
                } else {
                    anyhow::bail!("Unknown archive format");
                }
            } else if filename.ends_with("jar") {
                let filename_without_path = filename[filename.rfind('/').unwrap_or(0) + 1..].to_owned();
                let filename_without_ext = filename_without_path.trim_end_matches(".jar");

                if KtfEmulator::loadable_jar(buf) {
                    Box::new(KtfEmulator::from_jar(
                        platform,
                        &filename_without_path,
                        buf.to_vec(),
                        filename_without_ext,
                        filename_without_ext,
                        None,
                        options,
                    )?)
                } else if LgtEmulator::loadable_jar(buf) {
                    Box::new(LgtEmulator::from_jar(
                        platform,
                        &filename_without_path,
                        buf.to_vec(),
                        filename_without_ext,
                        filename_without_ext,
                        None,
                        options,
                    )?)
                } else if SktEmulator::loadable_jar(buf) {
                    Box::new(SktEmulator::from_jar(
                        platform,
                        &filename_without_path,
                        buf.to_vec(),
                        filename_without_ext,
                        None,
                    )?)
                } else {
                    Box::new(J2MEEmulator::from_jar(platform, &filename_without_path, buf.to_vec())?)
                }
            } else {
                anyhow::bail!("Unknown file format");
            };

            anyhow::Ok(Self {
                emulator,
                should_redraw,
                key_events: HashMap::new(),
                player,
            })
        })()
        .map_err(|e| JsError::new(&e.to_string()))
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
        self.player.set_volume(volume);
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
