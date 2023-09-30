use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_web::{performance_layer, MakeConsoleWriter};
use wasm_bindgen::prelude::*;

use wie::Wie;
use wie_backend::{Window, WindowCallbackEvent};
use wie_base::Event;
use wie_vendor_ktf::KtfArchive;

#[wasm_bindgen]
pub fn start(buf: &[u8]) {
    console_error_panic_hook::set_once();

    let fmt_layer = tracing_subscriber::fmt::layer().with_writer(MakeConsoleWriter);
    let perf_layer = performance_layer().with_details_from_fields(tracing_subscriber::fmt::format::Pretty::default());

    tracing_subscriber::registry().with(fmt_layer).with(perf_layer).init();

    let archive = KtfArchive::from_zip(buf).unwrap();

    let window = Window::new(240, 320); // TODO hardcoded size

    let mut wie = Wie::new(Box::new(archive), window.proxy()).unwrap();

    window.run(move |event| {
        match event {
            WindowCallbackEvent::Update => wie.tick()?,
            WindowCallbackEvent::Redraw => wie.send_event(Event::Redraw),
            _ => {}
        }

        anyhow::Ok(())
    })
}
