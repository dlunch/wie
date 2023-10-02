#![no_std]
extern crate alloc;

mod window;

use alloc::{boxed::Box, rc::Rc, string::ToString};
use core::cell::Cell;

use tracing_subscriber::{filter::LevelFilter, fmt::time::UtcTime, layer::SubscriberExt, util::SubscriberInitExt, Layer};
use tracing_web::{performance_layer, MakeConsoleWriter};
use wasm_bindgen::{prelude::*, JsError};
use web_sys::HtmlCanvasElement;

use wie_backend::{Archive, Backend, Executor};
use wie_base::Event;
use wie_vendor_ktf::KtfArchive;

use self::window::WindowImpl;

#[wasm_bindgen]
pub struct WieWeb {
    executor: Executor,
    backend: Backend,
    should_redraw: Rc<Cell<bool>>,
}

#[wasm_bindgen]
impl WieWeb {
    #[wasm_bindgen(constructor)]
    pub fn new(buf: &[u8], canvas: HtmlCanvasElement) -> Result<WieWeb, JsError> {
        (move || {
            let archive = KtfArchive::from_zip(buf)?;

            let should_redraw = Rc::new(Cell::new(true));
            let window = WindowImpl::new(canvas, should_redraw.clone());

            let mut backend = Backend::new(Box::new(window));

            let mut app = archive.load_app(&mut backend)?;
            let executor = Executor::new();

            app.start()?;

            anyhow::Ok(Self {
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

        self.executor.tick(&self.backend.time()).map_err(|e| JsError::new(&e.to_string()))
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_timer(UtcTime::rfc_3339())
        .with_writer(MakeConsoleWriter)
        .with_filter(LevelFilter::DEBUG);
    let perf_layer = performance_layer().with_details_from_fields(tracing_subscriber::fmt::format::Pretty::default());

    tracing_subscriber::registry().with(fmt_layer).with(perf_layer).init();
}
