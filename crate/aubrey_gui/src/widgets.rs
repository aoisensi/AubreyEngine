use aubrey_common::color::Rgba;

pub struct RootWidget;

pub struct PlaceholderWidget {
    pub color: Rgba,
}

impl Default for PlaceholderWidget {
    fn default() -> Self { Self { color: Rgba { r: 1.0, g: 0.0, b: 1.0, a: 1.0 } } }
}

pub enum BoxDirection { Horizontal, Vertical, HorizontalReverse, VerticalReverse }
pub struct BoxWidget { pub dir: BoxDirection }

