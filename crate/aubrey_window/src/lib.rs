use aubrey_core::app::{App, AppExit};
use aubrey_core::ecs::Entity;

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

// Window title text
#[derive(Clone, Default)]
pub struct WindowText(pub String);

// Window stats resource
pub struct WindowStats { pub open: usize }

// ---- Internal state ----
use std::cell::RefCell;
use std::collections::HashMap;

use winit::dpi::LogicalSize;
use winit::application::ApplicationHandler;
use winit::event::{WindowEvent, MouseButton, ElementState};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

// Global window maps for cross-crate access
thread_local! {
    static WIN_MAP: RefCell<(HashMap<Entity, Window>, HashMap<WindowId, Entity>)> = RefCell::new((HashMap::new(), HashMap::new()));
}

fn with_maps<R>(f: impl FnOnce(&mut HashMap<Entity, Window>, &mut HashMap<WindowId, Entity>) -> R) -> R {
    WIN_MAP.with(|cell| {
        let mut borrow = cell.borrow_mut();
        // avoid two simultaneous &mut borrows to fields by splitting
        let (map_ptr, rev_ptr): (*mut HashMap<Entity, Window>, *mut HashMap<WindowId, Entity>) = (&mut borrow.0, &mut borrow.1);
        unsafe { f(&mut *map_ptr, &mut *rev_ptr) }
    })
}

// Pending create requests collected by ECS system; consumed in about_to_wait where we have ActiveEventLoop
thread_local! { static PENDING_CREATES: RefCell<Vec<(Entity, WindowDescriptor)>> = RefCell::new(Vec::new()); }

// Redraw handler registered by GUI crate; invoked on resize/redraw
thread_local! { static REDRAW_HANDLER: RefCell<Option<fn(&mut App, Entity)>> = RefCell::new(None); }
// Mouse click handler and last known cursor positions per window
thread_local! { static CLICK_HANDLER: RefCell<Option<fn(&mut App, Entity, f32, f32)>> = RefCell::new(None); }
thread_local! { static CURSOR_POS: RefCell<HashMap<WindowId, (f32, f32)>> = RefCell::new(HashMap::new()); }

pub fn set_redraw_handler(f: Option<fn(&mut App, Entity)>) { REDRAW_HANDLER.with(|h| *h.borrow_mut() = f); }
pub fn set_click_handler(f: Option<fn(&mut App, Entity, f32, f32)>) { CLICK_HANDLER.with(|h| *h.borrow_mut() = f); }

pub fn with_window<R>(entity: Entity, f: impl FnOnce(&Window) -> R) -> Option<R> {
    WIN_MAP.with(|cell| cell.borrow().0.get(&entity).map(f))
}

pub fn window_size(entity: Entity) -> Option<(u32, u32)> {
    WIN_MAP.with(|cell| cell.borrow().0.get(&entity).map(|w| { let s = w.inner_size(); (s.width, s.height) }))
}

pub mod access { pub use super::{with_window as with_window_public, window_size as window_size_public}; }

// ---- Systems: only collect create requests ----
fn sys_collect_new_windows(ecs: &mut aubrey_core::ecs::ecs::Ecs) {
    let mut targets: Vec<(Entity, WindowDescriptor)> = Vec::new();
    ecs.for_each::<WindowDescriptor, _>(|e, desc| { if !ecs.has::<WindowCreated>(e) { targets.push((e, desc.clone())); } });
    if targets.is_empty() { return; }
    // Filter already created
    let existing: Vec<Entity> = WIN_MAP.with(|cell| cell.borrow().0.keys().copied().collect());
    PENDING_CREATES.with(|q| {
        let mut q = q.borrow_mut();
        for (e, d) in targets { if !existing.contains(&e) { q.push((e, d)); } }
    });
}

// ---- Application handler ----
struct Handler { app: App }

impl Handler {
    fn create_pending(&mut self, event_loop: &ActiveEventLoop) {
        let pending: Vec<(Entity, WindowDescriptor)> = PENDING_CREATES.with(|q| q.borrow_mut().drain(..).collect());
        if pending.is_empty() { return; }
        with_maps(|map, rev| {
            for (e, d) in pending {
                if map.contains_key(&e) { continue; }
                let attrs = WindowAttributes::default()
                    .with_title(d.title)
                    .with_inner_size(LogicalSize::new(d.width as f64, d.height as f64));
                let window = event_loop.create_window(attrs).expect("create window");
                let id = window.id();
                rev.insert(id, e);
                map.insert(e, window);
                self.app.insert_component(e, WindowCreated);
            }
        });
    }
}

impl ApplicationHandler for Handler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) { self.create_pending(event_loop); }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                with_maps(|map, rev| {
                    if let Some(&entity) = rev.get(&window_id) {
                        map.remove(&entity);
                        rev.remove(&window_id);
                    }
                });
            }
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                if let Some(entity) = WIN_MAP.with(|cell| cell.borrow().1.get(&window_id).copied()) {
                    if let Some(f) = REDRAW_HANDLER.with(|h| *h.borrow()) { f(&mut self.app, entity); }
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(entity) = WIN_MAP.with(|cell| cell.borrow().1.get(&window_id).copied()) {
                    if let Some(f) = REDRAW_HANDLER.with(|h| *h.borrow()) { f(&mut self.app, entity); }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                CURSOR_POS.with(|m| { m.borrow_mut().insert(window_id, (position.x as f32, position.y as f32)); });
            }
            WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Left, .. } => {
                if let Some(entity) = WIN_MAP.with(|cell| cell.borrow().1.get(&window_id).copied()) {
                    let (x, y) = CURSOR_POS.with(|m| m.borrow().get(&window_id).copied().unwrap_or((0.0, 0.0)));
                    if let Some(f) = CLICK_HANDLER.with(|h| *h.borrow()) { f(&mut self.app, entity, x, y); }
                    if let Some(f) = REDRAW_HANDLER.with(|h| *h.borrow()) { f(&mut self.app, entity); }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // run one ECS frame so systems can enqueue creates, update state, etc.
        self.app.update();
        // create windows that were requested by systems
        self.create_pending(event_loop);
        // update titles and request redraws
        WIN_MAP.with(|cell| {
            let maps = cell.borrow();
            for (e, w) in maps.0.iter() {
                if let Some(txt) = self.app.get_component::<WindowText>(*e) { w.set_title(&txt.0); }
                w.request_redraw();
            }
        });
        // publish stats and exit when no windows
        let open = WIN_MAP.with(|cell| cell.borrow().0.len());
        self.app.insert_resource(WindowStats { open });
        if open == 0 { self.app.insert_resource(AppExit); event_loop.exit(); }
        event_loop.set_control_flow(ControlFlow::Poll);
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {}
}

// Register window systems that only collect requests
pub fn register(app: &mut App) { app.add_systems(aubrey_core::ecs::Stage::Update, sys_collect_new_windows); }

// Entry that owns the event loop and drives the app
pub fn run(mut app: App) {
    // ensure our collector runs
    register(&mut app);
    let event_loop = EventLoop::new().expect("event loop");
    let mut handler = Handler { app };
    event_loop.run_app(&mut handler).expect("run app");
}
