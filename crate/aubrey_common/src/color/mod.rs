pub trait Color {
    fn rgba(&self) -> Rgba;
    #[inline]
    fn as_array(&self) -> [f32; 4] {
        let c = self.rgba();
        [c.r, c.g, c.b, c.a]
    }
}

impl Color for Rgba {
    fn rgba(&self) -> Rgba { *self }
}

pub mod rgba;
pub use rgba::Rgba;
