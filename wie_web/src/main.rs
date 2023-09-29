use wie::Wie;
use wie_backend::{Window, WindowCallbackEvent};
use wie_base::Event;
use wie_vendor_ktf::KtfArchive;

fn main() -> anyhow::Result<()> {
    let buf = Vec::new();

    let archive = KtfArchive::from_zip(&buf)?;

    let window = Window::new(240, 320); // TODO hardcoded size

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
