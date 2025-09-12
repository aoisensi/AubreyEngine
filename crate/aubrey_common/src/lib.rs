pub mod math;

pub mod color {
    #[derive(Clone, Copy, Debug, Default)]
    pub struct Rgba {
        pub r: f32,
        pub g: f32,
        pub b: f32,
        pub a: f32,
    }

    pub trait Color {
        fn rgba(&self) -> Rgba;
    }

    impl Color for Rgba {
        fn rgba(&self) -> Rgba { *self }
    }
}
