use std::io::stderr;

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

    #[cfg(target_arch = "wasm32")]
    unimplemented!();
    #[cfg(not(target_arch = "wasm32"))]
    wie_cli::start(&Args::parse().filename)
}
