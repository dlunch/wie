use std::{
    fs::File,
    io::{stderr, Read},
};

use clap::Parser;

use wie::Wie;
use wie_backend::{Window, WindowCallbackEvent};
use wie_vendor_ktf::KtfArchive;

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

    let archive = KtfArchive::from_zip(&buf)?;

    let window = Window::new(240, 320); // TODO hardcoded size

    let mut wie = Wie::new(Box::new(archive), window.proxy())?;

    window.run(move |event| {
        match event {
            WindowCallbackEvent::Update => wie.tick()?,
            WindowCallbackEvent::Event(x) => wie.send_event(x),
        }

        anyhow::Ok(())
    })
}
