mod core;
mod util;
mod wipi;

use std::{env, fs};

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let path = env::args().nth(1).ok_or_else(|| anyhow::anyhow!("No filename argument"))?;

    let data = fs::read(&path)?;
    let mut module = wipi::module::ktf::KtfWipiModule::new(&data, &path)?;

    module.start()?;

    Ok(())
}
