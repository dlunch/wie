extern crate alloc;

mod window;

use std::{
    fs::File,
    io::{stderr, Read},
};

use clap::Parser;
use tao::keyboard::KeyCode;

use wie_backend::{extract_zip, Archive, Backend, Executor};
use wie_base::{Event, WIPIKey};
use wie_vendor_ktf::KtfArchive;
use wie_vendor_lgt::LgtArchive;

use self::window::{WindowCallbackEvent, WindowImpl};

#[derive(Parser)]
struct Args {
    filename: String,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    let mut file = File::open(args.filename)?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let files = extract_zip(&buf)?;

    let archive: Box<dyn Archive> = if KtfArchive::is_ktf_archive(&files) {
        Box::new(KtfArchive::from_zip(files)?)
    } else if LgtArchive::is_lgt_archive(&files) {
        Box::new(LgtArchive::from_zip(files)?)
    } else {
        anyhow::bail!("Unknown archive format");
    };

    let window = WindowImpl::new(240, 320)?; // TODO hardcoded size

    let window_proxy = window.proxy();

    let mut backend = Backend::new(Box::new(window_proxy));
    let mut app = archive.load_app(&mut backend)?;

    let mut executor = Executor::new();
    app.start()?;

    window.run(move |event| {
        match event {
            WindowCallbackEvent::Update => executor
                .tick(&backend.time())
                .map_err(|x| anyhow::anyhow!("{}\n{}", x, app.crash_dump()))?,
            WindowCallbackEvent::Redraw => backend.push_event(Event::Redraw),
            WindowCallbackEvent::Keydown(x) => {
                if let Some(wipi_key) = convert_key(x) {
                    backend.push_event(Event::Keydown(wipi_key));
                }
            }
            WindowCallbackEvent::Keyup(x) => {
                if let Some(wipi_key) = convert_key(x) {
                    backend.push_event(Event::Keyup(wipi_key));
                }
            }
        }

        anyhow::Ok(())
    })
}

fn convert_key(key_code: KeyCode) -> Option<WIPIKey> {
    match key_code {
        KeyCode::Digit1 => Some(WIPIKey::NUM1),
        KeyCode::Digit2 => Some(WIPIKey::NUM2),
        KeyCode::Digit3 => Some(WIPIKey::NUM3),
        KeyCode::KeyQ => Some(WIPIKey::NUM4),
        KeyCode::KeyW => Some(WIPIKey::NUM5),
        KeyCode::KeyE => Some(WIPIKey::NUM6),
        KeyCode::KeyA => Some(WIPIKey::NUM7),
        KeyCode::KeyS => Some(WIPIKey::NUM8),
        KeyCode::KeyD => Some(WIPIKey::NUM9),
        KeyCode::KeyZ => Some(WIPIKey::STAR),
        KeyCode::KeyX => Some(WIPIKey::NUM0),
        KeyCode::KeyC => Some(WIPIKey::HASH),
        KeyCode::Space => Some(WIPIKey::FIRE),
        KeyCode::ArrowUp => Some(WIPIKey::UP),
        KeyCode::ArrowDown => Some(WIPIKey::DOWN),
        KeyCode::ArrowLeft => Some(WIPIKey::LEFT),
        KeyCode::ArrowRight => Some(WIPIKey::RIGHT),
        _ => None,
    }
}
