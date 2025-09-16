use aubrey_core::ecs::ecs::Ecs;
use aubrey_core::app::App;
use aubrey_core::ecs::{Children, Entity};
use crate::widgets::{PlaceholderWidget, BoxWidget, MarginComponent};
use aubrey_common::Direction;
use aubrey_render::PlaceholderItem;
use aubrey_common::color::Color;

pub fn compute_items_ecs(ecs: &Ecs, root: Entity, ww: u32, wh: u32) -> Vec<PlaceholderItem> {
    let mut items: Vec<PlaceholderItem> = Vec::new();
    fn layout_node(ecs: &Ecs, e: Entity, x: u32, y: u32, w: u32, h: u32, out: &mut Vec<PlaceholderItem>) {
        if let Some(ph) = ecs.get::<PlaceholderWidget>(e) {
            out.push(PlaceholderItem { x, y, w, h, color: ph.color.as_array(), thickness_px: 1.5 });
        }
        let children = ecs.get::<Children>(e).map(|c| c.0.clone()).unwrap_or_default();
        if children.is_empty() { return; }
        // apply margin to the area available to children
        let (x, y, w, h) = if let Some(m) = ecs.get::<MarginComponent>(e) {
            let ml = m.left.to_u32();
            let mr = m.right.to_u32();
            let mt = m.top.to_u32();
            let mb = m.bottom.to_u32();
            let nx = x.saturating_add(ml);
            let ny = y.saturating_add(mt);
            let nw = w.saturating_sub(ml.saturating_add(mr));
            let nh = h.saturating_sub(mt.saturating_add(mb));
            (nx, ny, nw, nh)
        } else { (x, y, w, h) };
        if let Some(bx) = ecs.get::<BoxWidget>(e) {
            let n = children.len() as u32; if n == 0 { return; }
            match bx.dir {
                Direction::Right | Direction::Start => {
                    let cw = w / n; let mut cx = x; let cy = y;
                    for &ch in &children { layout_node(ecs, ch, cx, cy, cw, h, out); cx += cw; }
                }
                Direction::Left | Direction::End => {
                    let cw = w / n; let mut cx = x; let cy = y;
                    for &ch in children.iter().rev() { layout_node(ecs, ch, cx, cy, cw, h, out); cx += cw; }
                }
                Direction::Down => {
                    let chh = h / n; let mut cy = y; let cx = x;
                    for &ch in &children { layout_node(ecs, ch, cx, cy, w, chh, out); cy += chh; }
                }
                Direction::Up => {
                    let chh = h / n; let mut cy = y; let cx = x;
                    for &ch in children.iter().rev() { layout_node(ecs, ch, cx, cy, w, chh, out); cy += chh; }
                }
            }
        } else {
            for &ch in &children { layout_node(ecs, ch, x, y, w, h, out); }
        }
    }
    layout_node(ecs, root, 0, 0, ww, wh, &mut items);
    items
}

pub fn compute_items_app(app: &App, root: Entity, ww: u32, wh: u32) -> Vec<PlaceholderItem> {
    let mut items: Vec<PlaceholderItem> = Vec::new();
    fn layout_node(app: &App, e: Entity, x: u32, y: u32, w: u32, h: u32, out: &mut Vec<PlaceholderItem>) {
        if let Some(ph) = app.get_component::<PlaceholderWidget>(e) {
            out.push(PlaceholderItem { x, y, w, h, color: ph.color.as_array(), thickness_px: 1.5 });
        }
        let children = app.get_component::<Children>(e).map(|c| c.0.clone()).unwrap_or_default();
        if children.is_empty() { return; }
        // apply margin to the area available to children
        let (x, y, w, h) = if let Some(m) = app.get_component::<MarginComponent>(e) {
            let ml = m.left.to_u32();
            let mr = m.right.to_u32();
            let mt = m.top.to_u32();
            let mb = m.bottom.to_u32();
            let nx = x.saturating_add(ml);
            let ny = y.saturating_add(mt);
            let nw = w.saturating_sub(ml.saturating_add(mr));
            let nh = h.saturating_sub(mt.saturating_add(mb));
            (nx, ny, nw, nh)
        } else { (x, y, w, h) };
        if let Some(bx) = app.get_component::<BoxWidget>(e) {
            let n = children.len() as u32; if n == 0 { return; }
            match bx.dir {
                Direction::Right | Direction::Start => {
                    let cw = w / n; let mut cx = x; let cy = y;
                    for &ch in &children { layout_node(app, ch, cx, cy, cw, h, out); cx += cw; }
                }
                Direction::Left | Direction::End => {
                    let cw = w / n; let mut cx = x; let cy = y;
                    for &ch in children.iter().rev() { layout_node(app, ch, cx, cy, cw, h, out); cx += cw; }
                }
                Direction::Down => {
                    let chh = h / n; let mut cy = y; let cx = x;
                    for &ch in &children { layout_node(app, ch, cx, cy, w, chh, out); cy += chh; }
                }
                Direction::Up => {
                    let chh = h / n; let mut cy = y; let cx = x;
                    for &ch in children.iter().rev() { layout_node(app, ch, cx, cy, w, chh, out); cy += chh; }
                }
            }
        } else {
            for &ch in &children { layout_node(app, ch, x, y, w, h, out); }
        }
    }
    layout_node(app, root, 0, 0, ww, wh, &mut items);
    items
}

// Hit testing: collect rectangles for entities that have PlaceholderWidget.
pub fn collect_hits_app(app: &App, root: Entity, ww: u32, wh: u32) -> Vec<(Entity, (u32, u32, u32, u32))> {
    let mut hits: Vec<(Entity, (u32, u32, u32, u32))> = Vec::new();
    fn walk(app: &App, e: Entity, x: u32, y: u32, w: u32, h: u32, out: &mut Vec<(Entity, (u32, u32, u32, u32))>) {
        if app.get_component::<PlaceholderWidget>(e).is_some() {
            out.push((e, (x, y, w, h)));
        }
        let children = app.get_component::<Children>(e).map(|c| c.0.clone()).unwrap_or_default();
        if children.is_empty() { return; }
        let (x, y, w, h) = if let Some(m) = app.get_component::<MarginComponent>(e) {
            let ml = m.left.to_u32();
            let mr = m.right.to_u32();
            let mt = m.top.to_u32();
            let mb = m.bottom.to_u32();
            let nx = x.saturating_add(ml);
            let ny = y.saturating_add(mt);
            let nw = w.saturating_sub(ml.saturating_add(mr));
            let nh = h.saturating_sub(mt.saturating_add(mb));
            (nx, ny, nw, nh)
        } else { (x, y, w, h) };
        if let Some(bx) = app.get_component::<BoxWidget>(e) {
            let n = children.len() as u32; if n == 0 { return; }
            match bx.dir {
                Direction::Right | Direction::Start => {
                    let cw = w / n; let mut cx = x; let cy = y;
                    for &ch in &children { walk(app, ch, cx, cy, cw, h, out); cx += cw; }
                }
                Direction::Left | Direction::End => {
                    let cw = w / n; let mut cx = x; let cy = y;
                    for &ch in children.iter().rev() { walk(app, ch, cx, cy, cw, h, out); cx += cw; }
                }
                Direction::Down => {
                    let chh = h / n; let mut cy = y; let cx = x;
                    for &ch in &children { walk(app, ch, cx, cy, w, chh, out); cy += chh; }
                }
                Direction::Up => {
                    let chh = h / n; let mut cy = y; let cx = x;
                    for &ch in children.iter().rev() { walk(app, ch, cx, cy, w, chh, out); cy += chh; }
                }
            }
        } else {
            for &ch in &children { walk(app, ch, x, y, w, h, out); }
        }
    }
    walk(app, root, 0, 0, ww, wh, &mut hits);
    hits
}
