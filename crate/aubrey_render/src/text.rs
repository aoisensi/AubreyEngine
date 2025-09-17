use ab_glyph::{FontRef, PxScale, point, Font, Glyph};

use crate::pack_rgba_u8;

// Embedded font accessor
pub fn noto_sans_regular() -> &'static [u8] { include_bytes!("../assets/NotoSans-Regular.ttf") }

/// Draw UTF-8 text into ARGB8888 buffer using ab_glyph (monochrome alpha blended)
pub fn draw_text_mono(buf: &mut [u32], width: usize, height: usize, stride: usize, x: i32, y: i32, text: &str, font_bytes: &[u8], px: f32, color: u32) {
    if let Ok(font) = FontRef::try_from_slice(font_bytes) {
        let scale = PxScale::from(px);
        let ascent = px; // rough baseline estimate; good enough for now
        let line_gap = px * 0.2;
        let mut caret = point(x as f32, y as f32 + ascent);
        for ch in text.chars() {
            if ch == '\n' { caret.x = x as f32; caret.y += px + line_gap; continue; }
            let id = font.glyph_id(ch);
            let sg = Glyph { id, scale, position: caret };
            if let Some(outline) = font.outline_glyph(sg) {
                let bb = outline.px_bounds();
                outline.draw(|gx, gy, cov| {
                    let gx = gx as i32 + bb.min.x as i32;
                    let gy = gy as i32 + bb.min.y as i32;
                    if gx >= 0 && gy >= 0 {
                        let ux = gx as usize; let uy = gy as usize;
                        if ux < width && uy < height {
                            let a = (cov.clamp(0.0, 1.0) * 255.0) as u8;
                            let src = color;
                            let sr = ((src >> 16) & 0xFF) as u8;
                            let sg = ((src >> 8) & 0xFF) as u8;
                            let sb = (src & 0xFF) as u8;
                            let dst = &mut buf[uy * stride + ux];
                            let dr = ((*dst >> 16) & 0xFF) as u8;
                            let dg = ((*dst >> 8) & 0xFF) as u8;
                            let db = (*dst & 0xFF) as u8;
                            let inv_a = 255u16 - a as u16;
                            let r = ((sr as u16 * a as u16 + dr as u16 * inv_a) / 255) as u8;
                            let g = ((sg as u16 * a as u16 + dg as u16 * inv_a) / 255) as u8;
                            let b = ((sb as u16 * a as u16 + db as u16 * inv_a) / 255) as u8;
                            *dst = pack_rgba_u8(r, g, b, 255);
                        }
                    }
                });
                let mut adv = (bb.max.x - bb.min.x).ceil();
                if adv <= 0.0 { adv = px * 0.6; }
                caret.x += adv + (px * 0.1);
            } else {
                // Fallback advance
                caret.x += px * 0.6;
            }
        }
    }
}

