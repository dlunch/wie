# AGENTS.md

## Build/Test/Lint Commands
- **Build**: `cargo build` (default member: `wie`)
- **Test all**: `cargo test --workspace`
- **Test single**: `cargo test -p wie-ktf test_helloworld` or `cargo test -p <crate> <test_name>`
- **Lint**: `cargo clippy --workspace`
- **Format**: `cargo fmt` (uses rustfmt.toml: max_width=150, use_field_init_shorthand=true)
- **Web install**: `npm install`
- **Web build**: `npm run build:dev` or `npm run build:prod`
- **Web dev server**: `npm start`
- **Pre-commit (MANDATORY)**: Always run `cargo fmt` and `cargo clippy --workspace` before every commit. CI will reject unformatted or lint-failing code.

## Code Style Guidelines
- **Edition**: Rust 2024
- **no_std**: Most crates are `#![no_std]` with `extern crate alloc`
- **Imports**: Group by source (std/alloc → external crates → local crate → workspace crates), alphabetized
- **Error handling**: Use `wie_util::Result<T>` / `WieError` enum. Propagate with `?`, no panics in library code
- **Naming**: snake_case for functions/variables, PascalCase for types, SCREAMING_CASE for constants
- **Types**: Explicit types preferred. Never use `as any` equivalents or suppress errors
- **Async**: Use `async-trait` for async trait methods

## Engineering Principles
- Keep implementations and automation minimal. Do not add options, dependencies, scripts, metadata, workflow steps, or explicit version/retention settings unless they are required for the requested behavior; rely on established tool and repository defaults when they are sufficient.
- Avoid redundant or defensive validation for states already guaranteed by internal types, trusted workflow context, build tools, or a following command that will fail naturally. Add validation only at meaningful external/dynamic boundaries or when it provides required observable behavior.
- Keep emulated runtime state authoritative in guest memory. Do not add host-side state or metadata registries; host adapters may only reference and operate on guest-backed structures.
- Do not add special-case branches keyed to a specific application or Java class in shared runtime infrastructure. Represent confirmed ABI differences as data and handle them through generic mechanisms.
- Use reference implementations only to understand architectural boundaries and responsibilities. Derive API contracts from public specifications and documentation, and implement behavior independently without copying reference implementation code or algorithms.

## Project Layout
- `wie-backend`: System-level services for APIs
- `wie`: CLI for local testing
- `wie-core-arm`: ARM emulation
- `wie-jvm-support`: JVM support
- `wie-midp`, `wie-wipi-*`, `wie-skvm`: API implementations
- `wie-j2me`, `wie-skt`, `wie-ktf`, `wie-lgt`: Platform-specific logic
- `wie-web`: Rust/WebAssembly and TypeScript web frontend
- `wie-app`: Desktop and mobile Tauri frontend
