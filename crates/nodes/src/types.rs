use mech3ax_types::PrimitiveEnum;

pub const ZONE_DEFAULT: u32 = 255;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PrimitiveEnum)]
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
