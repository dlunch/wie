use std::{
    fs::File,
    io::{stderr, Read},
};

use clap::Parser;

#[derive(Parser)]
struct Args {
    filename: String,
    main_class_name: Option<String>,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    let mut file = File::open(args.filename)?;

    let mut buf = vec![0; file.metadata()?.len() as usize];
    file.read_to_end(&mut buf)?;

    wie::start(&buf, args.main_class_name)?;

    Ok(())
}
