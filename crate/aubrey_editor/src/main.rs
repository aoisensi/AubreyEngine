use aubrey_core::app::App;
use aubrey_window::{WindowDescriptor, WindowText};
use aubrey_gui::{RootWidget, PlaceholderWidget};
use aubrey_common::color::Rgba;
use aubrey_core::ecs::Children;

fn main() {
    let mut app = App::new();

    // window systems are registered by aubrey_window::run

    let e = app.spawn_one(WindowDescriptor::new("Aubrey Editor", 640, 400));
    app.insert_component(e, WindowText("Aubrey Editor".into()));

    // GUI root and placeholder widget as children of window
    let root = app.spawn_one(RootWidget);
    let placeholder = app.spawn_one(PlaceholderWidget { color: Rgba { r: 1.0, g: 0.0, b: 1.0, a: 1.0 } });
    app.insert_component(e, Children(vec![root, placeholder]));

    // register gui rendering
    aubrey_gui::register(&mut app);
    aubrey_window::run(app);
}
