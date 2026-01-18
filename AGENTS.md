# AGENTS.md

## Build/Test/Lint Commands
- **Build**: `cargo build` (default member: `wie_cli`)
- **Test all**: `cargo test --workspace`
- **Test single**: `cargo test -p wie_ktf test_helloworld` or `cargo test -p <crate> <test_name>`
- **Lint**: `cargo clippy --workspace`
- **Format**: `cargo fmt` (uses rustfmt.toml: max_width=150, use_field_init_shorthand=true)

## Code Style Guidelines
- **Edition**: Rust 2024
- **no_std**: Most crates are `#![no_std]` with `extern crate alloc`
- **Imports**: Group by source (std/alloc → external crates → local crate → workspace crates), alphabetized
- **Error handling**: Use `wie_util::Result<T>` / `WieError` enum. Propagate with `?`, no panics in library code
- **Naming**: snake_case for functions/variables, PascalCase for types, SCREAMING_CASE for constants
- **Types**: Explicit types preferred. Never use `as any` equivalents or suppress errors
- **Async**: Use `async-trait` for async trait methods

## Project Layout
- `wie_backend`: System-level services for APIs
- `wie_cli`: CLI for local testing
- `wie_core_arm`: ARM emulation
- `wie_jvm_support`: JVM support
- `wie_midp`, `wie_wipi_*`, `wie_skvm`: API implementations
- `wie_j2me`, `wie_skt`, `wie_ktf`, `wie_lgt`: Platform-specific logic
