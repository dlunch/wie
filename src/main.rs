extern crate alloc;

cfg_if::cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        mod cli;

        fn main() -> anyhow::Result<()> {
            cli::run()
        }
    } else {
        fn main() -> anyhow::Result<()> {
            Ok(())
        }
    }
}
