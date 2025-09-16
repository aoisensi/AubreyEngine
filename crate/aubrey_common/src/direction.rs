// Common direction enum shared across crates
// Used by GUI layout BoxWidget, etc.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    Start,
    End,
}
