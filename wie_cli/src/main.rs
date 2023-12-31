extern crate alloc;

mod window;

use std::{fs, io::stderr};

use clap::Parser;

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

fn convert_key(key_code: u32) -> Option<KeyCode> {
    match key_code {
        2 => Some(KeyCode::NUM1),
        3 => Some(KeyCode::NUM2),
        4 => Some(KeyCode::NUM3),
        16 => Some(KeyCode::NUM4), // Q
        17 => Some(KeyCode::NUM5), // W
        18 => Some(KeyCode::NUM6), // E
        30 => Some(KeyCode::NUM7), // A
        31 => Some(KeyCode::NUM8), // S
        32 => Some(KeyCode::NUM9), // D
        44 => Some(KeyCode::STAR), // Z
        45 => Some(KeyCode::NUM0), // X
        46 => Some(KeyCode::HASH), // C
        57 => Some(KeyCode::OK),   // Space
        103 => Some(KeyCode::UP),
        108 => Some(KeyCode::DOWN),
        105 => Some(KeyCode::LEFT),
        106 => Some(KeyCode::RIGHT),
        _ => None,
    }
}
