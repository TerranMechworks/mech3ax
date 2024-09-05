use mech3ax_types::primitive_enum;

pub(crate) const ZONE_DEFAULT: u32 = 255;

primitive_enum! {
    pub(crate) enum NodeType: u32 {
        Empty = 0,
        Camera = 1,
        World = 2,
        Window = 3,
        Display = 4,
        Object3d = 5,
        LoD = 6,
        Light = 9,
    }
}
