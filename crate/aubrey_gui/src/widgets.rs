use aubrey_common::color::Rgba;
use aubrey_common::{Direction, Size};
use aubrey_core::app::App;
use aubrey_core::ecs::Entity;

pub struct RootWidget;

pub struct PlaceholderWidget {
    pub color: Rgba,
}

impl Default for PlaceholderWidget {
    fn default() -> Self { Self { color: Rgba { r: 1.0, g: 0.0, b: 1.0, a: 1.0 } } }
}

pub struct BoxWidget { pub dir: Direction }

pub struct MarginComponent {
    pub left: Size,
    pub right: Size,
    pub top: Size,
    pub bottom: Size,
}

impl MarginComponent {
    pub fn all(size: Size) -> Self { Self { left: size, right: size, top: size, bottom: size } }
    pub fn vertical(size: Size) -> Self { Self { left: Size::ZERO, right: Size::ZERO, top: size, bottom: size } }
    pub fn horizontal(size: Size) -> Self { Self { left: size, right: size, top: Size::ZERO, bottom: Size::ZERO } }
}

pub struct MouseActionComponent {
    pub on_click: Option<fn(&mut App, Entity)>,
    pub on_down: Option<fn(&mut App, Entity)>,
    pub on_up: Option<fn(&mut App, Entity)>,
    pub on_enter: Option<fn(&mut App, Entity)>,
    pub on_leave: Option<fn(&mut App, Entity)>,
}

impl Default for MouseActionComponent {
    fn default() -> Self {
        Self { on_click: None, on_down: None, on_up: None, on_enter: None, on_leave: None }
    }
}
