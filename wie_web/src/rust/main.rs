#![no_std]
extern crate alloc;

use alloc::{boxed::Box, string::ToString};

use tracing_subscriber::{filter::LevelFilter, fmt::time::UtcTime, layer::SubscriberExt, util::SubscriberInitExt, Layer};
use tracing_web::{performance_layer, MakeConsoleWriter};
use wasm_bindgen::{prelude::*, JsError};

use wie::Wie;
use wie_backend::{Window, WindowCallbackEvent};
use wie_base::Event;
use wie_vendor_ktf::KtfArchive;

fn do_start(buf: &[u8]) -> anyhow::Result<()> {
    let archive = KtfArchive::from_zip(buf)?;

    let window = Window::new(240, 320)?; // TODO hardcoded size

    let mut wie = Wie::new(Box::new(archive), window.proxy())?;

    window.run(move |event| {
        match event {
            WindowCallbackEvent::Update => wie.tick()?,
            WindowCallbackEvent::Redraw => wie.send_event(Event::Redraw),
            _ => {}
        }

        anyhow::Ok(())
    })
}

#[wasm_bindgen]
pub fn start(buf: &[u8]) -> Result<(), JsError> {
    do_start(buf).map_err(|e| JsError::new(&e.to_string()))
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
