#[cfg(not(target_arch = "wasm32"))]

fn main() -> anyhow::Result<()> {
    use std::io::stderr;

    use clap::Parser;

    use wie::start;

    #[derive(Parser)]
    struct Args {
        filename: String,
    }

    tracing_subscriber::fmt()
        .with_writer(stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    start(&args.filename)
}

#[cfg(target_arch = "wasm32")]
fn main() {}
