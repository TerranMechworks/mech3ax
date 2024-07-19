use num_derive::FromPrimitive;

pub const ZONE_DEFAULT: u32 = 255;

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
#[repr(u32)]
pub enum NodeType {
    Empty = 0,
    Camera = 1,
    World = 2,
    Window = 3,
    Display = 4,
    Object3d = 5,
    LoD = 6,
    Light = 9,
}
