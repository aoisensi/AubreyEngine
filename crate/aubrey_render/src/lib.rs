use aubrey_common::color::Color;
use aubrey_core::ecs::Entity;
use softbuffer::{Context, Surface};
use aubrey_window::access::{with_window_public as with_window};
use std::num::NonZeroU32;
use winit::window::Window;

fn with_surface<R>(win: Entity, f: impl FnOnce(&mut Surface<&Window, &Window>, (u32, u32)) -> R) -> Option<R> {
    with_window(win, |wnd| {
        let sz = wnd.inner_size();
        let (w, h) = (sz.width, sz.height);
        let ctx = Context::new(wnd).ok()?;
        let mut surf = Surface::new(&ctx, wnd).ok()?;
        let _ = surf.resize(NonZeroU32::new(w).unwrap(), NonZeroU32::new(h).unwrap());
        Some(f(&mut surf, (w, h)))
    }).flatten()
}
/// Execute a closure with a CPU frame buffer for the given window entity.
/// The closure receives: (buf, width, height, stride). Buffer format: ARGB8888.
pub fn with_frame(win: Entity, f: impl FnOnce(&mut [u32], usize, usize, usize)) -> Option<()> {
    with_surface(win, |surf, (wpx, hpx)| {
        let mut buf = match surf.buffer_mut() { Ok(b) => b, Err(_) => return (), };
        let width = wpx as usize;
        let height = hpx as usize;
        let stride = width; // tightly packed
        {
            let slice: &mut [u32] = &mut buf;
            f(slice, width, height, stride);
        }
        let _ = buf.present();
    })
}

#[inline]
pub fn pack_rgba_u8(r: u8, g: u8, b: u8, a: u8) -> u32 {
    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

#[inline]
pub fn pack_color<C: Color>(c: &C) -> u32 {
    let rgba = c.rgba();
    let r = (rgba.r.clamp(0.0, 1.0) * 255.0) as u8;
    let g = (rgba.g.clamp(0.0, 1.0) * 255.0) as u8;
    let b = (rgba.b.clamp(0.0, 1.0) * 255.0) as u8;
    let a = (rgba.a.clamp(0.0, 1.0) * 255.0) as u8;
    pack_rgba_u8(r, g, b, a)
}

#[inline]
pub fn clear(buf: &mut [u32], width: usize, height: usize, stride: usize, color: u32) {
    for y in 0..height {
        let row = &mut buf[y * stride..y * stride + width];
        for px in row.iter_mut() { *px = color; }
    }
}

#[inline]
pub fn put_pixel(buf: &mut [u32], width: usize, height: usize, stride: usize, x: i32, y: i32, color: u32) {
    if x >= 0 && y >= 0 {
        let ux = x as usize; let uy = y as usize;
        if ux < width && uy < height {
            buf[uy * stride + ux] = color;
        }
    }
}

/// Bresenham's line algorithm
pub fn draw_line(buf: &mut [u32], width: usize, height: usize, stride: usize, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: u32) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        put_pixel(buf, width, height, stride, x0, y0, color);
        if x0 == x1 && y0 == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x0 += sx; }
        if e2 <= dx { err += dx; y0 += sy; }
    }
}

pub fn draw_rect_outline(buf: &mut [u32], width: usize, height: usize, stride: usize, x: i32, y: i32, w: i32, h: i32, color: u32) {
    if w <= 0 || h <= 0 { return; }
    let x0 = x;
    let y0 = y;
    let x1 = x + w - 1;
    let y1 = y + h - 1;
    // top/bottom
    draw_line(buf, width, height, stride, x0, y0, x1, y0, color);
    draw_line(buf, width, height, stride, x0, y1, x1, y1, color);
    // left/right
    draw_line(buf, width, height, stride, x0, y0, x0, y1, color);
    draw_line(buf, width, height, stride, x1, y0, x1, y1, color);
}
