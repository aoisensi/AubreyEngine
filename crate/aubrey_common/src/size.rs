#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Size {
    ZERO,
    Px(f32),
}

impl Size {
    #[inline]
    pub fn to_u32(self) -> u32 {
        match self {
            Size::ZERO => 0,
            Size::Px(v) => {
                if v.is_sign_negative() { 0 } else { v.floor() as u32 }
            }
        }
    }
}

