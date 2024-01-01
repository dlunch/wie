extern crate alloc;

mod window;

use std::{fs, io::stderr};

use clap::Parser;
use winit::keyboard::{KeyCode as WinitKeyCode, PhysicalKey};

use wie_backend::{extract_zip, Archive, Backend, Executor, Platform, Window};
use wie_base::{Event, KeyCode};
use wie_ktf::KtfArchive;
use wie_lgt::LgtArchive;
use wie_skt::SktArchive;

use self::window::{WindowCallbackEvent, WindowImpl};

struct WieCliPlatform {
    window: WindowImpl,
}

impl WieCliPlatform {
    fn new() -> Self {
        Self {
            window: WindowImpl::new(240, 320).unwrap(), // TODO hardcoded size
        }
    }
}

impl Platform for WieCliPlatform {
    fn create_window(&self) -> Box<dyn Window> {
        Box::new(self.window.proxy())
    }
}

#[derive(Parser)]
struct Args {
    filename: String,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    start(&Args::parse().filename)
}

pub fn start(filename: &str) -> anyhow::Result<()> {
    let buf = fs::read(filename)?;

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

    let mut platform = WieCliPlatform::new();

    let mut backend = Backend::new(&archive.id(), &mut platform);
    let mut app = archive.load_app(&mut backend)?;

    let mut executor = Executor::new();
    app.start()?;

    platform.window.run(move |event| {
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

fn convert_key(key: PhysicalKey) -> Option<KeyCode> {
    match key {
        PhysicalKey::Code(WinitKeyCode::Digit1) => Some(KeyCode::NUM1),
        PhysicalKey::Code(WinitKeyCode::Digit2) => Some(KeyCode::NUM2),
        PhysicalKey::Code(WinitKeyCode::Digit3) => Some(KeyCode::NUM3),
        PhysicalKey::Code(WinitKeyCode::KeyQ) => Some(KeyCode::NUM4),
        PhysicalKey::Code(WinitKeyCode::KeyW) => Some(KeyCode::NUM5),
        PhysicalKey::Code(WinitKeyCode::KeyE) => Some(KeyCode::NUM6),
        PhysicalKey::Code(WinitKeyCode::KeyA) => Some(KeyCode::NUM7),
        PhysicalKey::Code(WinitKeyCode::KeyS) => Some(KeyCode::NUM8),
        PhysicalKey::Code(WinitKeyCode::KeyD) => Some(KeyCode::NUM9),
        PhysicalKey::Code(WinitKeyCode::KeyZ) => Some(KeyCode::STAR),
        PhysicalKey::Code(WinitKeyCode::KeyX) => Some(KeyCode::NUM0),
        PhysicalKey::Code(WinitKeyCode::KeyC) => Some(KeyCode::HASH),
        PhysicalKey::Code(WinitKeyCode::Space) => Some(KeyCode::OK),
        PhysicalKey::Code(WinitKeyCode::ArrowUp) => Some(KeyCode::UP),
        PhysicalKey::Code(WinitKeyCode::ArrowDown) => Some(KeyCode::DOWN),
        PhysicalKey::Code(WinitKeyCode::ArrowLeft) => Some(KeyCode::LEFT),
        PhysicalKey::Code(WinitKeyCode::ArrowRight) => Some(KeyCode::RIGHT),
        _ => None,
    }
}
