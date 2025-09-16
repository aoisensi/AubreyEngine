# Completion Checklist

- Build: `cargo build` succeeds for the workspace.
- Format: `cargo fmt --all` applied (no diffs or committed).
- Lint: `cargo clippy --workspace --all-targets -- -D warnings` clean.
- Tests: `cargo test` pass.
- Runtime sanity (if touching GUI/window/render/core): `cargo run -p aubrey_editor` opens a window; title updates; placeholder rendering visible; closing window exits.
- Docs: Updated `README.md` and/or `docs/*.md` if API surface or behavior changed.
- API review: New public items follow naming conventions and are re-exported appropriately.
- Scheduling: If adding systems, verify stage, label uniqueness, `before/after` dependencies.
- ECS safety: Check component/resource types are `'static + Send + Sync`; avoid borrow conflicts in systems.
