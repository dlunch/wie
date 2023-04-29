mod core;
mod util;
mod wipi;

use std::{env, fs};

fn main() {
    pretty_env_logger::init();

    let path = env::args().nth(1).unwrap();

    let data = fs::read(&path).unwrap();
    let mut module = wipi::module::ktf::KtfWipiModule::new(&data, &path);

    module.start();
}
