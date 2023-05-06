mod core;
mod util;
mod wipi;

use std::{env, fs};

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let path = env::args().nth(1).ok_or_else(|| anyhow::anyhow!("No filename argument"))?;
    let main_class = env::args().nth(2).ok_or_else(|| anyhow::anyhow!("No main_class argument"))?;

    let data = fs::read(&path)?;
    let module = wipi::module::ktf::KtfWipiModule::new(&data, &path, &main_class)?;

    module.start()?;

    Ok(())
}
