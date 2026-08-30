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
    collections::BTreeMap,
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

use wie_backend::{Emulator, Event, Font, Instant, KeyCode, Options, Platform, Screen, extract_zip};
use wie_j2me::J2MEEmulator;
use wie_ktf::KtfEmulator;
use wie_lgt::LgtEmulator;
use wie_skt::SktEmulator;

use self::{audio_sink::AudioSink, database::DatabaseRepository, filesystem::WebFilesystem, window::WindowImpl};

enum ArchivePlatform {
    Ktf,
    Lgt,
    Skt,
}

fn parse_archive(buf: &[u8]) -> anyhow::Result<(ArchivePlatform, BTreeMap<String, Vec<u8>>)> {
    let files = extract_zip(buf)?;

    if !files.keys().any(|name| name.to_ascii_lowercase().ends_with(".jar")) {
        anyhow::bail!("Archive does not contain a JAR file");
    }

    let platform = if KtfEmulator::loadable_archive(&files) {
        ArchivePlatform::Ktf
    } else if LgtEmulator::loadable_archive(&files) {
        ArchivePlatform::Lgt
    } else if SktEmulator::loadable_archive(&files) {
        ArchivePlatform::Skt
    } else {
        anyhow::bail!("Unknown archive format");
    };

    Ok((platform, files))
}

fn jar_app_id<'a>(filename: &'a str, buf: &[u8]) -> &'a str {
    if KtfEmulator::loadable_jar(buf) || LgtEmulator::loadable_jar(buf) || SktEmulator::loadable_jar(buf) {
        &filename[..filename.len() - 4]
    } else {
        filename
    }
}

struct WieWebPlatform {
    database_repository: DatabaseRepository,
    filesystem: WebFilesystem,
    font: Font,
    window: WindowImpl,
}

// XXX we're on single thread
unsafe impl Sync for WieWebPlatform {}
unsafe impl Send for WieWebPlatform {}

impl WieWebPlatform {
    fn new(window: WindowImpl, font: Font) -> Self {
        Self {
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
        Box::new(AudioSink::new())
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
    let lowercase_filename = filename.to_ascii_lowercase();
    let metadata = if lowercase_filename.ends_with(".zip") {
        let (platform, files) = parse_archive(buf).map_err(|error| JsError::new(&error.to_string()))?;
        match platform {
            ArchivePlatform::Ktf => KtfEmulator::archive_id(&files)
                .zip(KtfEmulator::archive_title(&files))
                .map(|(id, title)| (id, title, KtfEmulator::archive_icon(&files))),
            ArchivePlatform::Lgt => LgtEmulator::archive_id(&files)
                .zip(LgtEmulator::archive_title(&files))
                .map(|(id, title)| (id, title, LgtEmulator::archive_icon(&files))),
            ArchivePlatform::Skt => SktEmulator::archive_id(&files)
                .zip(SktEmulator::archive_title(&files))
                .map(|(id, title)| (id, title, SktEmulator::archive_icon(&files))),
        }
    } else if lowercase_filename.ends_with(".jar") {
        let filename = filename.rsplit('/').next().unwrap();
        J2MEEmulator::jar_metadata(buf)
            .map_err(|error| JsError::new(&error.to_string()))?
            .map(|(title, icon)| (jar_app_id(filename, buf).to_owned(), title, icon))
    } else {
        return Err(JsError::new("Unknown file format"));
    };
    let (id, title, icon) = metadata.ok_or_else(|| JsError::new("App metadata does not contain an ID, title or entry point"))?;

    Ok(ImportedAppMetadata {
        id,
        title,
        icon: icon.unwrap_or_default(),
    })
}

#[wasm_bindgen]
impl WieWeb {
    #[wasm_bindgen(constructor)]
    pub fn new(filename: &str, buf: &[u8], canvas: HtmlCanvasElement, font_data: Vec<u8>) -> Result<WieWeb, JsError> {
        (move || {
            let should_redraw = Arc::new(AtomicBool::new(true));
            let window = WindowImpl::new(canvas, should_redraw.clone());
            let font = Font::try_from_vec(font_data)?;
            let platform = Box::new(WieWebPlatform::new(window, font));
            let options = Options {
                enable_gdbserver: false,
                profile: None,
            };

            let emulator: Box<dyn Emulator> = if filename.to_ascii_lowercase().ends_with(".zip") {
                let (archive_platform, files) = parse_archive(buf)?;

                match archive_platform {
                    ArchivePlatform::Ktf => Box::new(KtfEmulator::from_archive(platform, files, options)?),
                    ArchivePlatform::Lgt => Box::new(LgtEmulator::from_archive(platform, files, options)?),
                    ArchivePlatform::Skt => Box::new(SktEmulator::from_archive(platform, files)?),
                }
            } else if filename.to_ascii_lowercase().ends_with(".jar") {
                let filename_without_path = filename.rsplit('/').next().unwrap().to_owned();
                let app_id = jar_app_id(&filename_without_path, buf);

                if KtfEmulator::loadable_jar(buf) {
                    Box::new(KtfEmulator::from_jar(
                        platform,
                        &filename_without_path,
                        buf.to_vec(),
                        app_id,
                        app_id,
                        None,
                        options,
                    )?)
                } else if LgtEmulator::loadable_jar(buf) {
                    Box::new(LgtEmulator::from_jar(
                        platform,
                        &filename_without_path,
                        buf.to_vec(),
                        app_id,
                        app_id,
                        None,
                        options,
                    )?)
                } else if SktEmulator::loadable_jar(buf) {
                    Box::new(SktEmulator::from_jar(platform, &filename_without_path, buf.to_vec(), app_id, None)?)
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
