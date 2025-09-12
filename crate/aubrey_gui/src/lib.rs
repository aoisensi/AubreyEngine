use aubrey_common::color::{Rgba, Color};
use aubrey_core::app::{App, Stage};
use aubrey_core::ecs::{Children, Entity};
use aubrey_render as render;
use aubrey_window;
use aubrey_core::ecs::ecs::Ecs;

pub struct RootWidget;

pub struct PlaceholderWidget<C: Color = Rgba> {
    pub color: C,
}

impl Default for PlaceholderWidget<Rgba> {
    fn default() -> Self { Self { color: Rgba { r: 1.0, g: 0.0, b: 1.0, a: 1.0 } } }
}

fn render_one(ecs: &mut Ecs, w: Entity) {
    // find GUI root and placeholder under the window
    let mut has_root = false;
    let mut placeholder_col: Option<u32> = None;

    if let Some(children) = ecs.get::<Children>(w) {
        for c in &children.0 {
            if ecs.has::<RootWidget>(*c) { has_root = true; }
            if let Some(ph) = ecs.get::<PlaceholderWidget>(*c) {
                let color = render::pack_color(&ph.color);
                placeholder_col = Some(color);
            }
        }
    }
    if !has_root { return; }
    let Some(col) = placeholder_col else { return };

    let _ = render::with_frame(w, |buf, width, height, stride| {
        // clear to black
        render::clear(buf, width, height, stride, 0xFF000000);

        // border
        render::draw_rect_outline(buf, width, height, stride, 0, 0, width as i32, height as i32, col);
        // diagonals (corner to corner regardless of aspect ratio)
        render::draw_line(buf, width, height, stride, 0, 0, width as i32 - 1, height as i32 - 1, col);
        render::draw_line(buf, width, height, stride, 0, height as i32 - 1, width as i32 - 1, 0, col);
    });
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
        // Extract needed data via App public API
        let mut has_root = false;
        let mut col_opt: Option<u32> = None;
        if let Some(children) = app.get_component::<Children>(w) {
            for c in &children.0 {
                if app.get_component::<RootWidget>(*c).is_some() { has_root = true; }
                if let Some(ph) = app.get_component::<PlaceholderWidget>(*c) {
                    let color = render::pack_color(&ph.color);
                    col_opt = Some(color);
                }
            }
        }
        if !has_root { return; }
        let Some(col) = col_opt else { return };
        let _ = render::with_frame(w, |buf, width, height, stride| {
            render::clear(buf, width, height, stride, 0xFF000000);
            render::draw_rect_outline(buf, width, height, stride, 0, 0, width as i32, height as i32, col);
            render::draw_line(buf, width, height, stride, 0, 0, width as i32 - 1, height as i32 - 1, col);
            render::draw_line(buf, width, height, stride, 0, height as i32 - 1, width as i32 - 1, 0, col);
        });
    }
    aubrey_window::set_redraw_handler(Some(render_one_app));
    app.add_systems(Stage::Last, sys_gui_render);
}
