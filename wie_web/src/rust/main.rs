use wasm_bindgen::prelude::*;

use wie::Wie;
use wie_backend::{Window, WindowCallbackEvent};
use wie_base::Event;
use wie_vendor_ktf::KtfArchive;

#[wasm_bindgen]
pub fn start(buf: &[u8]) {
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
