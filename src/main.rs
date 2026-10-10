extern crate alloc;

mod cli;

fn main() -> anyhow::Result<()> {
    cli::run()
}
