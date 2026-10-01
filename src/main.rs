extern crate alloc;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        mod audio_sink;
        mod cli;
        mod database;
        mod filesystem;
        mod window;

        fn main() -> anyhow::Result<()> {
            cli::run()
        }
    } else {
        fn main() -> anyhow::Result<()> {
            Ok(())
        }
    }
}
