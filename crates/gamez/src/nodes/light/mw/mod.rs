mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::gamez::nodes::Light;
use mech3ax_api_types::{AffineMatrix, Color, Count32, Vec3};
use mech3ax_types::{Maybe, Offsets, Ptr, bitflags, impl_as_bytes};
pub(crate) use read::read;
pub(crate) use write::write;

pub(crate) fn size(light: &Light) -> u32 {
    let parent_size = (light.parent_indices.len() as u32) * 4;
    use mech3ax_types::AsBytes as _;
    LightMwC::SIZE.wrapping_add(parent_size)
}

bitflags! {
    struct LightFlags: u32 {
        const RECALC            = 1 << 0; // 0x001
        const UNK1              = 1 << 1; // 0x002
        const DIRECTIONAL       = 1 << 2; // 0x004
        const DIRECTED_SOURCE   = 1 << 3; // 0x008
        const POINT_SOURCE      = 1 << 4; // 0x010
        const SATURATED         = 1 << 5; // 0x020
        const SUBDIVIDE         = 1 << 6; // 0x040
        const STATIC            = 1 << 7; // 0x080
        const COLOR             = 1 << 8; // 0x100
        const UNK9              = 1 << 9; // 0x200
    }
}

type Flags = Maybe<u32, LightFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct LightMwC {
    orientation: Vec3,         // 000
    translate: Vec3,           // 012
    euler_angles: Vec3,        // 024
    world_translate: Vec3,     // 036
    transform: AffineMatrix,   // 048
    world_point_source: Vec3,  // 096
    field108: Vec3,            // 108
    camera_point_source: Vec3, // 120
    world_view_vector: Vec3,   // 132
    camera_view_vector: Vec3,  // 144
    diffuse: f32,              // 156
    ambient: f32,              // 160
    color: Color,              // 164
    flags: Flags,              // 176
    range_near: f32,           // 180
    range_far: f32,            // 184
    range_near_sq: f32,        // 188
    range_far_sq: f32,         // 192
    range_inv: f32,            // 196
    parent_count: Count32,     // 200
    parent_ptr: Ptr,           // 204
}
impl_as_bytes!(LightMwC, 208);

const WORLD_VIEW: Vec3 = Vec3 {
    x: 0.0,
    y: 1.0,
    z: 0.0,
};
