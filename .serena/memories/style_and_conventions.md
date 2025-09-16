# Style and Conventions

- Language/edition: Rust (workspace). Crates use editions 2021 or 2024 as declared.
- Naming: Standard Rust conventions (`snake_case` for items, `CamelCase` for types/traits). Public re-exports to shape API surface (`pub use`).
- Modules: Small, focused modules: `ecs::{ecs, schedule, query, ...}`; re-export common items at `ecs` mod root.
- Safety: No `unsafe` in current code; keep code safe and simple.
- API surface: Bevy-like `App` with fluent methods. Avoid exposing internals of `Ecs` where possible; prefer accessor methods.
- Scheduling: Prefer `add_systems_with_label`/`add_systems_with_deps` to control order. Keep labels unique within a stage.
- Commands: Use per-stage `Commands` for deferred entity ops inside systems; direct `Ecs::insert` allowed for immediate setup.
- Query: Prefer `query()`/`query2()` and filters (`With<T>`, `Without<T>`) instead of manual store iteration.
- GUI/render: Keep rendering helpers stateless and pure; avoid long-lived globals except thread-local window maps provided by window crate.
- Docs: Add examples to code doc comments when useful; keep design docs in `docs/`.
