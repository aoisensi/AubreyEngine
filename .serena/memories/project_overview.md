# AubreyEngine — Project Overview

- Purpose: Minimal Rust game engine with an ECS core and a small GUI/window stack. Emphasizes data locality (chunked by stage/context), type-separated contexts (`Component<C>`), and per-context resources/time/scheduling.
- Tech stack: Rust workspace with multiple crates. Windowing via `winit`, CPU rendering via `softbuffer` (with placeholders for future Vulkan backend). GUI demo draws simple lines/rectangles.
- OS: Linux (dev), should be portable where `winit`/`softbuffer` support.

## Crates
- `aubrey_core` (lib): ECS, resources, scheduler, Bevy-like `App` entry, queries, commands, dynamic registry.
- `aubrey_common` (lib): Common types (e.g., `color::Rgba`, math).
- `aubrey_window` (lib): Window system integration (collect create requests in ECS, winit event loop owner). Components: `WindowDescriptor`, `WindowText`, `WindowCreated`. Resources: `WindowStats`.
- `aubrey_render` (lib): CPU frame buffer rendering helpers (`with_frame`, `clear`, `draw_line`, `draw_rect_outline`), color packing.
- `aubrey_gui` (lib): Simple GUI scaffold. Components `RootWidget`, `PlaceholderWidget`. System renders placeholder into window surfaces.
- `aubrey_editor` (bin): Example/editor entrypoint spawning a window, root GUI, and registering GUI+window systems.

## Key Concepts
- ECS: `Entity` is numeric ID; components stored per type; resources are type-unique global values.
- Systems: `FnMut(&mut Ecs)` trait objects scheduled into stages.
- Stages: `PreStartup, Startup, PostStartup, First, PreUpdate, Update, PostUpdate, Last` with order and label dependency (`before/after`).
- Commands: Per-stage deferred ops (`spawn/insert/despawn`), applied at stage end.
- Queries: `Ecs::query<T>()`, `Ecs::query2<A,B>()` with filters `With<T>`/`Without<T>`.
- Dynamic API: `Registry` maps string names to `ComponentId`/`ResourceId` for script-friendly access.

## Running
- Editor demo: `cargo run -p aubrey_editor` (opens a window titled "Aubrey Editor" with placeholder GUI and simple CPU rendering). Requires `winit` and `softbuffer` runtime deps.

## Docs
- `docs/ecs.md`: ECS API and usage.
- `docs/scheduling.md`: Stage/order/label-based scheduling details with example.
- `docs/components/tick.md`: Design for `TickComponent<C>` and `Time<C>` (planned/outlined).

## Repo Layout
- `crate/*` — workspace member crates (see above).
- `README.md` — high-level summary and doc pointers.
- `docs/` — design docs.
