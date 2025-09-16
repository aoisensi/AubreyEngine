use aubrey_core::app::{App, Stage};
use aubrey_core::ecs::{Children, Entity};
use aubrey_render as render;
use aubrey_core::fs::Vfs;
use aubrey_window;
use aubrey_core::ecs::ecs::Ecs;

pub mod widgets;
pub mod layout;

pub use widgets::{RootWidget, PlaceholderWidget, BoxWidget, MarginComponent, MouseActionComponent};
pub use aubrey_common::{Direction, Size};

use std::sync::atomic::{AtomicU64, Ordering};
static RNG_SEED: AtomicU64 = AtomicU64::new(0);

fn seed_rng_once() {
    if RNG_SEED.load(Ordering::Relaxed) == 0 {
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0);
        let tid = std::thread::current().id();
        // poor man's thread id hash
        let tid_hash = unsafe { std::mem::transmute::<std::thread::ThreadId, u64>(tid) };
        let seed = nanos ^ tid_hash ^ 0x9E3779B97F4A7C15u64;
        RNG_SEED.store(seed, Ordering::Relaxed);
    }
}

fn rand_f32() -> f32 {
    seed_rng_once();
    let mut s = RNG_SEED.load(Ordering::Relaxed);
    // LCG
    s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
    RNG_SEED.store(s, Ordering::Relaxed);
    let v = ((s >> 32) as u32) as f32 / (u32::MAX as f32);
    v
}

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
        let vfs = app.resource::<Vfs>();
        let _ = render::with_frame(w, |buf, width, height, stride| {
            render::clear(buf, width, height, stride, render::pack_rgba_u8(0,0,0,255));
            for it in &items {
                let c = it.color;
                let r = (c[0].clamp(0.0,1.0) * 255.0) as u8;
                let g = (c[1].clamp(0.0,1.0) * 255.0) as u8;
                let b = (c[2].clamp(0.0,1.0) * 255.0) as u8;
                let a = (c[3].clamp(0.0,1.0) * 255.0) as u8;
                let col = render::pack_rgba_u8(r,g,b,a);
                render::draw_rect_outline(buf, width, height, stride, it.x as i32, it.y as i32, it.w as i32, it.h as i32, col);
            }
            // Draw TextLabel nodes
            if let Some(vfs) = vfs {
                let mut labels: Vec<(Entity, (u32,u32,u32,u32))> = layout::collect_textlabels_app(app, root, ww, wh);
                for (e, (x,y,_w,_h)) in labels.drain(..) {
                    if let Some(lbl) = app.get_component::<widgets::TextLabel>(e) {
                        if let Some(bytes) = vfs.read(&lbl.font_path) {
                            let color = render::pack_rgba_u8(
                                (lbl.color.r.clamp(0.0,1.0) * 255.0) as u8,
                                (lbl.color.g.clamp(0.0,1.0) * 255.0) as u8,
                                (lbl.color.b.clamp(0.0,1.0) * 255.0) as u8,
                                (lbl.color.a.clamp(0.0,1.0) * 255.0) as u8,
                            );
                            render::draw_text_mono(buf, width, height, stride, x as i32 + 4, y as i32 + 4, &lbl.text, &bytes, lbl.size_px, color);
                        }
                    }
                }
            }
        });
    }
    fn on_click_app(app: &mut App, w: Entity, x: f32, y: f32) {
        // Find root under window
        let mut root: Option<Entity> = None;
        if let Some(children) = app.get_component::<Children>(w) {
            for c in &children.0 { if app.get_component::<RootWidget>(*c).is_some() { root = Some(*c); break; } }
        }
        let Some(root) = root else { return };
        let (ww, wh) = match aubrey_window::window_size(w) { Some((w, h)) => (w as u32, h as u32), None => return };
        // walk and collect clickable rects
        let hits = layout::collect_hits_app(app, root, ww, wh);
        // pick the last that contains the point (deepest)
        let xi = x as u32; let yi = y as u32;
        let mut target: Option<Entity> = None;
        for (e, (rx, ry, rw, rh)) in hits.into_iter() {
            if xi >= rx && yi >= ry && xi < rx.saturating_add(rw) && yi < ry.saturating_add(rh) {
                target = Some(e);
            }
        }
        if let Some(e) = target {
            if let Some(ph) = app.get_component_mut::<widgets::PlaceholderWidget>(e) {
                // generate a random vivid-ish color
                let r = rand_f32();
                let g = rand_f32();
                let b = rand_f32();
                ph.color = aubrey_common::color::Rgba { r, g, b, a: 1.0 };
            }
        }
    }

    aubrey_window::set_redraw_handler(Some(render_one_app));
    aubrey_window::set_click_handler(Some(on_click_app));
    app.add_systems(Stage::Last, sys_gui_render);
}
