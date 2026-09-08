#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    wie::run()
}

#[cfg(target_arch = "wasm32")]
fn main() -> anyhow::Result<()> {
    Ok(())
}
