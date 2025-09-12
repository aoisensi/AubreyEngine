use aubrey_core::app::{App, Stage, AppExit};
use aubrey_core::ecs::{Entity};

// Public components
#[derive(Clone)]
pub struct WindowDescriptor {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

impl WindowDescriptor {
    pub fn new(title: impl Into<String>, width: u32, height: u32) -> Self {
        Self { title: title.into(), width, height }
    }
}

// Marker component indicating a native window was created for this entity
pub struct WindowCreated;

// Example "data to display" component. For今はウィンドウのタイトルに反映する。
#[derive(Clone, Default)]
pub struct WindowText(pub String);

// 公開リソース: 現在開いているウィンドウ数
pub struct WindowStats {
    pub open: usize,
}

// ---- Internal state (not in ECS resources to avoid Send/Sync bounds) ----
use std::cell::RefCell;
use std::collections::HashMap;

use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::event_loop::ControlFlow;
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::{Window, WindowBuilder, WindowId};

struct WinState {
    event_loop: Option<EventLoop<()>>, // kept as Option to enable run_return each frame
    // entity <-> window mapping
    windows_by_entity: HashMap<Entity, Window>,
    entities_by_id: HashMap<WindowId, Entity>,
}

impl WinState {
    fn new() -> Self { Self { event_loop: Some(EventLoop::new()), windows_by_entity: HashMap::new(), entities_by_id: HashMap::new() } }
}

thread_local! {
    static WIN_STATE: RefCell<WinState> = RefCell::new(WinState::new());
}

fn with_state<R>(f: impl FnOnce(&mut WinState) -> R) -> R {
    WIN_STATE.with(|cell| {
        let mut borrow = cell.borrow_mut();
        f(&mut *borrow)
    })
}

// ---- Systems ----

// Create native windows for entities that have a WindowDescriptor but no WindowCreated
fn sys_create_windows(ecs: &mut aubrey_core::ecs::ecs::Ecs) {
    // Collect target entities first to avoid borrow issues
    let mut targets: Vec<(Entity, WindowDescriptor)> = Vec::new();
    ecs.for_each::<WindowDescriptor, _>(|e, desc| {
        if !ecs.has::<WindowCreated>(e) { targets.push((e, desc.clone())); }
    });

    if targets.is_empty() { return; }

    with_state(|st| {
        for (e, desc) in targets {
            // Skip if already created (double-check against state)
            if st.windows_by_entity.contains_key(&e) { continue; }

            let ev = st.event_loop.as_ref().expect("event_loop missing");
            // Builder API on winit 0.28
            let window = WindowBuilder::new()
                .with_title(desc.title)
                .with_inner_size(LogicalSize::new(desc.width as f64, desc.height as f64))
                .build(ev)
                .expect("failed to create window");

            let id = window.id();
            st.entities_by_id.insert(id, e);
            st.windows_by_entity.insert(e, window);

            // Mark entity as created
            ecs.insert::<WindowCreated>(e, WindowCreated);
        }
    });
}

// Poll events and apply simple behavior: close requests despawn marker; update title from WindowText
fn sys_poll_events(ecs: &mut aubrey_core::ecs::ecs::Ecs) {
    with_state(|st| {
        let mut to_close: Vec<Entity> = Vec::new();

        // Drain one round of events
        let mut ev = st.event_loop.take().expect("event_loop missing");
        ev.run_return(|event, _target, control| {
            *control = ControlFlow::Poll;
            match event {
                Event::WindowEvent { event, window_id } => {
                    match event {
                        WindowEvent::CloseRequested => {
                            if let Some(&e) = st.entities_by_id.get(&window_id) { to_close.push(e); }
                        }
                        _ => {}
                    }
                }
                Event::MainEventsCleared => {
                    *control = ControlFlow::Exit;
                }
                _ => {}
            }
        });
        st.event_loop = Some(ev);

        // Update titles from WindowText
        let mut title_updates: Vec<(Entity, String)> = Vec::new();
        ecs.for_each::<WindowText, _>(|e, txt| { title_updates.push((e, txt.0.clone())); });
        for (e, t) in title_updates {
            if let Some(w) = st.windows_by_entity.get(&e) { w.set_title(&t); }
        }

        // Handle close requests: drop native window and remove marker
        for e in to_close {
            if let Some(w) = st.windows_by_entity.remove(&e) {
                st.entities_by_id.remove(&w.id());
            }
            // Remove marker so it can be recreated if needed
            ecs.for_each_mut::<WindowCreated, _>(|ent, _marker| {
                if ent == e { /* just to drive mutable borrow; removal via dyn API not present */ }
            });
            // Currently Ecs lacks a remove<T>(), so we leave the marker in place.
            // If the entity is despawned elsewhere, state map cleanup keeps us safe.
        }
    });
}

// Publish window stats as a Resource
fn sys_publish_stats(ecs: &mut aubrey_core::ecs::ecs::Ecs) {
    let open = with_state(|st| st.windows_by_entity.len());
    ecs.insert_resource(WindowStats { open });
    if open == 0 { ecs.insert_resource(AppExit); }
}

// Public helper to register window systems
pub fn register(app: &mut App) {
    app
        .add_systems(Stage::Startup, sys_create_windows)
        .add_systems(Stage::Update, sys_create_windows)
        .add_systems(Stage::Update, sys_poll_events)
        .add_systems(Stage::Last, sys_publish_stats);
}

// (no extra public helpers for now)

