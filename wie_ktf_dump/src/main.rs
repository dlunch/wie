use std::{fs, path::PathBuf};

use clap::Parser;

/// Dump a KTF game's relocated client.bin for static analysis.
///
/// Loads the archive, runs the self-relocation entry at IMAGE_BASE+1, and
/// writes the resulting memory window to disk. Load the output in IDA / Ghidra
/// at base 0x100000.
#[derive(Parser)]
struct Args {
    /// KTF game zip archive.
    input: PathBuf,
    /// Output file (raw bytes, no header).
    output: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let zip = fs::read(&args.input)?;
    let image = futures::executor::block_on(wie_ktf::dump_image(&zip)).map_err(|e| anyhow::anyhow!("{e}"))?;
    fs::write(&args.output, &image)?;
    Ok(())
}
