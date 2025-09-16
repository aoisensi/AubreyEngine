use aubrey_core::app::App;
use aubrey_window::{WindowDescriptor, WindowText};
use aubrey_gui::{RootWidget, PlaceholderWidget, BoxWidget, Direction, MarginComponent, Size};
use aubrey_gui::widgets::TextLabel;
use aubrey_common::color::Rgba;
use aubrey_core::ecs::Children;
use aubrey_core::fs::{Vfs, MemBackend};

fn main() {
    let mut app = App::new();

    // window systems are registered by aubrey_window::run

    let e = app.spawn_one(WindowDescriptor::new("Aubrey Editor", 640, 400));
    app.insert_component(e, WindowText("Aubrey Editor".into()));

    // Initialize VFS with in-memory root and mount at "/"
    let mut vfs = Vfs::new();
    vfs.mount("/", Box::new(MemBackend::new()));
    // Ensure /editor/fonts exists
    let _ = vfs.mkdir("/editor");
    let _ = vfs.mkdir("/editor/fonts");
    // Place embedded NotoSans-Regular.ttf into /editor/fonts/
    let font_path = "/editor/fonts/NotoSans-Regular.ttf";
    if !vfs.exists(font_path) {
        let bytes = aubrey_render::noto_sans_regular();
        let _ = vfs.write(font_path, bytes);
    }
    app.insert_resource(vfs);

    // GUI root and 2x2 grid using nested BoxWidget (Vertical -> two Horizontal rows)
    let root = app.spawn_one(RootWidget);
    let top_box = app.spawn_one(BoxWidget { dir: Direction::Down });

    // rows
    let row_top = app.spawn_one(BoxWidget { dir: Direction::Right });
    let row_bottom = app.spawn_one(BoxWidget { dir: Direction::Right });

    // placeholders
    let ph1 = app.spawn_one(PlaceholderWidget { color: Rgba { r: 1.0, g: 0.0, b: 0.0, a: 1.0 } }); // red
    let ph2 = app.spawn_one(PlaceholderWidget { color: Rgba { r: 0.0, g: 1.0, b: 0.0, a: 1.0 } }); // green
    let ph3 = app.spawn_one(PlaceholderWidget { color: Rgba { r: 0.0, g: 0.0, b: 1.0, a: 1.0 } }); // blue
    let ph4 = app.spawn_one(PlaceholderWidget { color: Rgba { r: 1.0, g: 1.0, b: 0.0, a: 1.0 } }); // yellow

    // wrap each placeholder with a margin container to create spacing inside cells
    let ph1_wrap = app.spawn_one(MarginComponent::all(Size::Px(8.0)));
    let ph2_wrap = app.spawn_one(MarginComponent::all(Size::Px(8.0)));
    let ph3_wrap = app.spawn_one(MarginComponent::all(Size::Px(8.0)));
    let ph4_wrap = app.spawn_one(MarginComponent::all(Size::Px(8.0)));
    app.insert_component(ph1_wrap, Children(vec![ph1]));
    app.insert_component(ph2_wrap, Children(vec![ph2]));
    app.insert_component(ph3_wrap, Children(vec![ph3]));
    app.insert_component(ph4_wrap, Children(vec![ph4]));

    // rows contain the wrapped placeholders
    app.insert_component(row_top, Children(vec![ph1_wrap, ph2_wrap]));
    app.insert_component(row_bottom, Children(vec![ph3_wrap, ph4_wrap]));

    // wrap rows with vertical margins to create spacing between rows
    let row_top_wrap = app.spawn_one(MarginComponent::vertical(Size::Px(8.0)));
    let row_bottom_wrap = app.spawn_one(MarginComponent::vertical(Size::Px(8.0)));
    app.insert_component(row_top_wrap, Children(vec![row_top]));
    app.insert_component(row_bottom_wrap, Children(vec![row_bottom]));

    // top_box contains wrapped rows
    app.insert_component(top_box, Children(vec![row_top_wrap, row_bottom_wrap]));

    // Add a TextLabel in top-left cell
    let label = app.spawn_one(TextLabel { text: "Hello, Aubrey!".into(), color: Rgba { r: 0.9, g: 0.9, b: 0.9, a: 1.0 }, font_path: font_path.into(), size_px: 18.0 });
    app.insert_component(ph1, Children(vec![label]));

    // add an outer margin around everything under root
    let root_margin = app.spawn_one(MarginComponent::all(Size::Px(16.0)));
    app.insert_component(root_margin, Children(vec![top_box]));
    app.insert_component(root, Children(vec![root_margin]));
    app.insert_component(e, Children(vec![root]));

    // register gui rendering
    aubrey_gui::register(&mut app);
    aubrey_window::run(app);
}
