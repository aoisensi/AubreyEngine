use aubrey_core::app::{App, Stage};
use aubrey_core::ecs::{Children, Entity};
use aubrey_render as render;
use aubrey_window;
use aubrey_core::ecs::ecs::Ecs;

pub mod widgets;
pub mod layout;

pub use widgets::{RootWidget, PlaceholderWidget, BoxWidget, MarginComponent, MouseActionComponent};
pub use aubrey_common::{Direction, Size};

fn render_one(ecs: &mut Ecs, w: Entity) {
    // find GUI root under the window
    let mut root: Option<Entity> = None;
    if let Some(children) = ecs.get::<Children>(w) {
        for c in &children.0 { if ecs.has::<RootWidget>(*c) { root = Some(*c); break; } }
    }
    let Some(root) = root else { return };

    let (ww, wh) = match aubrey_window::window_size(w) { Some((w, h)) => (w as u32, h as u32), None => return };
    let items = layout::compute_items_ecs(ecs, root, ww, wh);
    if items.is_empty() { return; }
    let _ = render::render_placeholders_wgpu(w, &items);
}

fn sys_gui_render(ecs: &mut Ecs) {
    // collect windows
    let mut windows: Vec<Entity> = Vec::new();
    ecs.for_each::<aubrey_window::WindowCreated, _>(|e, _| windows.push(e));

    for w in windows {
        render_one(ecs, w);
    }
}

pub fn register(app: &mut App) {
    // Immediate redraw handler during resize / redraw-request using App API
    fn render_one_app(app: &mut App, w: Entity) {
        // Find root under window
        let mut root: Option<Entity> = None;
        if let Some(children) = app.get_component::<Children>(w) {
            for c in &children.0 { if app.get_component::<RootWidget>(*c).is_some() { root = Some(*c); break; } }
        }
        let Some(root) = root else { return };

        let (ww, wh) = match aubrey_window::window_size(w) { Some((w, h)) => (w as u32, h as u32), None => return };
        let items = layout::compute_items_app(app, root, ww, wh);
        if items.is_empty() { return; }
        let _ = render::render_placeholders_wgpu(w, &items);
    }
    aubrey_window::set_redraw_handler(Some(render_one_app));
    app.add_systems(Stage::Last, sys_gui_render);
}
