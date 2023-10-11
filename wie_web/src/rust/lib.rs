#![no_std]
extern crate alloc;

mod window;

use alloc::{boxed::Box, format, rc::Rc, string::ToString};
use core::cell::Cell;

use tracing_subscriber::{filter::LevelFilter, fmt::time::UtcTime, layer::SubscriberExt, util::SubscriberInitExt, Layer};
use tracing_web::MakeConsoleWriter;
use wasm_bindgen::{prelude::*, JsError};
use web_sys::HtmlCanvasElement;

use wie_backend::{extract_zip, App, Archive, Backend, Executor};
use wie_base::Event;
use wie_vendor_ktf::KtfArchive;
use wie_vendor_lgt::LgtArchive;
use wie_vendor_skt::SktArchive;

use self::window::WindowImpl;

#[wasm_bindgen]
pub struct WieWeb {
    app: Box<dyn App>,
    executor: Executor,
    backend: Backend,
    should_redraw: Rc<Cell<bool>>,
}

#[wasm_bindgen]
impl WieWeb {
    #[wasm_bindgen(constructor)]
    pub fn new(buf: &[u8], canvas: HtmlCanvasElement) -> Result<WieWeb, JsError> {
        (move || {
            let files = extract_zip(buf)?;

            let archive: Box<dyn Archive> = if KtfArchive::is_ktf_archive(&files) {
                Box::new(KtfArchive::from_zip(files)?)
            } else if LgtArchive::is_lgt_archive(&files) {
                Box::new(LgtArchive::from_zip(files)?)
            } else if SktArchive::is_skt_archive(&files) {
                Box::new(SktArchive::from_zip(files)?)
            } else {
                anyhow::bail!("Unknown archive format");
            };

            let should_redraw = Rc::new(Cell::new(true));
            let window = WindowImpl::new(canvas, should_redraw.clone());

            let mut backend = Backend::new(Box::new(window));

            let mut app = archive.load_app(&mut backend)?;
            let executor = Executor::new();

            app.start()?;

            anyhow::Ok(Self {
                app,
                executor,
                backend,
                should_redraw,
            })
        })()
        .map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn update(&mut self) -> Result<(), JsError> {
        if self.should_redraw.get() {
            self.backend.push_event(Event::Redraw);
            self.should_redraw.set(false);
        }

        self.executor.tick(&self.backend.time()).map_err(|e| {
            let error_str = format!("{}\n{}", e, self.app.crash_dump());

            JsError::new(&error_str)
        })
    }

    pub fn send_key(&mut self, key: i32) -> Result<(), JsError> {
        let key = unsafe { core::mem::transmute(key) }; // TODO

        self.backend.push_event(Event::Keydown(key));
        self.backend.push_event(Event::Keyup(key));

        Ok(())
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_timer(UtcTime::rfc_3339())
        .with_writer(MakeConsoleWriter)
        .with_filter(LevelFilter::DEBUG);

    tracing_subscriber::registry().with(fmt_layer).init();
}
