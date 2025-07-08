mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::{AffineMatrix, Color, Vec3};
use mech3ax_types::{bitflags, impl_as_bytes, Maybe, Offsets, Ptr};
pub(crate) use read::read;
pub(crate) use write::write;

bitflags! {
    struct LightFlags: u32 {
        const RECALC            = 1 <<  0; // 0x001
        const UNK1              = 1 <<  1; // 0x002
        const DIRECTIONAL       = 1 <<  2; // 0x004
        const DIRECTED_SOURCE   = 1 <<  3; // 0x008
        const POINT_SOURCE      = 1 <<  4; // 0x010
        const SATURATED         = 1 <<  5; // 0x020
        const SUBDIVIDE         = 1 <<  6; // 0x040
        const LIGHT_MAP         = 1 <<  7; // 0x080
        const STATIC            = 1 <<  8; // 0x100
        const COLOR             = 1 <<  9; // 0x200
        const BICOLORED         = 1 << 10; // 0x400
        const UNK9              = 1 << 11; // 0x800
    }
}

type Flags = Maybe<u32, LightFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct LightPmC {
    orientation: Vec3,          // 000
    translate: Vec3,            // 012
    euler_angles: Vec3,         // 024
    world_translate: Vec3,      // 036
    transform: AffineMatrix,    // 048
    world_point_source: Vec3,   // 096
    field108: Vec3,             // 108
    camera_point_source: Vec3,  // 120
    world_view_vector: Vec3,    // 132
    camera_view_vector: Vec3,   // 144
    diffuse: f32,               // 156
    ambient: f32,               // 160
    color: Color,               // 164
    color_ambient: Color,       // 176
    color_diffuse_mixed: Color, // 188
    color_ambient_mixed: Color, // 200
    color_da_combined: Color,   // 212
    flags: Flags,               // 224
    range_near: f32,            // 228
    range_far: f32,             // 232
    range_near_sq: f32,         // 236
    range_far_sq: f32,          // 240
    range_inv: f32,             // 244
    parent_count: i32,          // 248
    parent_ptr: Ptr,            // 252
}
impl_as_bytes!(LightPmC, 256);

const WORLD_VIEW: Vec3 = Vec3 {
    x: 0.0,
    y: 1.0,
    z: 0.0,
};
