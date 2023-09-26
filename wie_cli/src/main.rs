use std::{
    fs::File,
    io::{stderr, Read},
};

use clap::Parser;

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

    wie::start(&buf)?;

    Ok(())
}
