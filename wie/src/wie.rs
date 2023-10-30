extern crate alloc;

cfg_if::cfg_if! {
if #[cfg(not(target_arch = "wasm32"))] {
mod window;

use std::{
    fs::File,
    io::{stderr, Read},
};

use clap::Parser;
use tao::keyboard::KeyCode as TAOKeyCode;

use wie_backend::{extract_zip, Archive, Backend, Executor};
use wie_base::{Event, KeyCode};
use wie_ktf::KtfArchive;
use wie_lgt::LgtArchive;
use wie_skt::SktArchive;

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
    } else if SktArchive::is_skt_archive(&files) {
        Box::new(SktArchive::from_zip(files)?)
    } else {
        anyhow::bail!("Unknown archive format");
    };

    let window = WindowImpl::new(240, 320)?; // TODO hardcoded size

    let window_proxy = window.proxy();

    let mut backend = Backend::new(&archive.id(), Box::new(window_proxy));
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
                if let Some(keycode) = convert_key(x) {
                    backend.push_event(Event::Keydown(keycode));
                }
            }
            WindowCallbackEvent::Keyup(x) => {
                if let Some(keycode) = convert_key(x) {
                    backend.push_event(Event::Keyup(keycode));
                }
            }
        }

        anyhow::Ok(())
    })
}

fn convert_key(key_code: TAOKeyCode) -> Option<KeyCode> {
    match key_code {
        TAOKeyCode::Digit1 => Some(KeyCode::NUM1),
        TAOKeyCode::Digit2 => Some(KeyCode::NUM2),
        TAOKeyCode::Digit3 => Some(KeyCode::NUM3),
        TAOKeyCode::KeyQ => Some(KeyCode::NUM4),
        TAOKeyCode::KeyW => Some(KeyCode::NUM5),
        TAOKeyCode::KeyE => Some(KeyCode::NUM6),
        TAOKeyCode::KeyA => Some(KeyCode::NUM7),
        TAOKeyCode::KeyS => Some(KeyCode::NUM8),
        TAOKeyCode::KeyD => Some(KeyCode::NUM9),
        TAOKeyCode::KeyZ => Some(KeyCode::STAR),
        TAOKeyCode::KeyX => Some(KeyCode::NUM0),
        TAOKeyCode::KeyC => Some(KeyCode::HASH),
        TAOKeyCode::Space => Some(KeyCode::OK),
        TAOKeyCode::ArrowUp => Some(KeyCode::UP),
        TAOKeyCode::ArrowDown => Some(KeyCode::DOWN),
        TAOKeyCode::ArrowLeft => Some(KeyCode::LEFT),
        TAOKeyCode::ArrowRight => Some(KeyCode::RIGHT),
        _ => None,
    }
}
}
else {
fn main() {}
}
}
