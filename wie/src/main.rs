extern crate alloc;

mod window;

use std::{
    fs::File,
    io::{stderr, Read},
};

use clap::Parser;

use wie_backend::{Archive, Backend, Executor};
use wie_base::{Event, WIPIKey};
use wie_vendor_ktf::KtfArchive;

use self::window::{WindowCallbackEvent, WindowImpl};

#[derive(Parser)]
struct Args {
    filename: String,
}

static KEY_MAP: phf::Map<u32, WIPIKey> = phf::phf_map! {
    2u32 => WIPIKey::NUM1,
    3u32 => WIPIKey::NUM2,
    4u32 => WIPIKey::NUM3,
    16u32 => WIPIKey::NUM4, // Q
    17u32 => WIPIKey::NUM5, // W
    18u32 => WIPIKey::NUM6, // E
    30u32 => WIPIKey::NUM7, // A
    31u32 => WIPIKey::NUM8, // S
    32u32 => WIPIKey::NUM9, // D
    44u32 => WIPIKey::STAR, // Z
    45u32 => WIPIKey::NUM0, // X
    46u32 => WIPIKey::HASH, // C
    57u32 => WIPIKey::FIRE, // Space
    103u32 => WIPIKey::UP,
    108u32 => WIPIKey::DOWN,
    105u32 => WIPIKey::LEFT,
    106u32 => WIPIKey::RIGHT,
};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    let mut file = File::open(args.filename)?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let archive = KtfArchive::from_zip(&buf)?;

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
                if let Some(entry) = KEY_MAP.get_entry(&x) {
                    backend.push_event(Event::Keydown(*entry.1));
                }
            }
            WindowCallbackEvent::Keyup(x) => {
                if let Some(entry) = KEY_MAP.get_entry(&x) {
                    backend.push_event(Event::Keyup(*entry.1));
                }
            }
        }

        anyhow::Ok(())
    })
}
