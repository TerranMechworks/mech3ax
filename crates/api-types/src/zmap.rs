//! Recoil `m*.zmap` data structures.
use crate::{fld, Vec3};
use mech3ax_types::impl_as_bytes;

fld! {
    #[repr(C)]
    struct MapColor : Val {
        r: u8,
        g: u8,
        b: u8,
    }
}
impl_as_bytes!(MapColor, 3);

fld! {
    struct MapFeature {
        color: MapColor,
        vertices: Vec<Vec3>,
        objective: i32,
    }
}

fld! {
    struct Zmap {
        unk04: u32,
        min: Vec3,
        max: Vec3,
        features: Vec<MapFeature>,
    }
}
