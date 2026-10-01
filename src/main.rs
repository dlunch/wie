extern crate alloc;

#[cfg(not(target_arch = "wasm32"))]
mod cli;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    cli::run()
}

#[cfg(target_arch = "wasm32")]
fn main() -> anyhow::Result<()> {
    Ok(())
}
