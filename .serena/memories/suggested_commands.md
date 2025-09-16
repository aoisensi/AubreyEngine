# Suggested Commands

## Build and Run
- Build workspace: `cargo build`
- Run editor demo: `cargo run -p aubrey_editor`
- Run with backtraces (debugging): `RUST_BACKTRACE=1 cargo run -p aubrey_editor`

## Test
- Run tests (workspace): `cargo test`
- Specific crate tests: `cargo test -p aubrey_core`

## Lint & Format
- Lint with clippy (all targets, all features): `cargo clippy --workspace --all-targets -- -D warnings`
- Format: `cargo fmt --all`

## Useful Dev
- List crates: `cargo metadata --no-deps -q`
- Grep code fast: `rg PATTERN`
- Show file tree: `fd -t f | sort` (or `find . -type f`)
- Run a single bin with args: `cargo run -p aubrey_editor -- ARGS...`

## OS/Deps
- Linux packages may be required for `winit`/`softbuffer` depending on distro (X11/Wayland dev libs). If runtime fails, consult those crates' docs.
