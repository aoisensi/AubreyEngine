use aubrey_core::app::App;
use aubrey_window::{register, WindowDescriptor, WindowText};

fn main() {
    let mut app = App::new();

    register(&mut app);

    let e = app.spawn_one(WindowDescriptor::new("Aubrey Editor", 1024, 640));
    app.insert_component(e, WindowText("Aubrey Editor".into()));

    app.run();
}
