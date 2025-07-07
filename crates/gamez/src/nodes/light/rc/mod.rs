mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::gamez::nodes::Light;
use mech3ax_api_types::{AffineMatrix, Color, Vec3};
use mech3ax_types::{impl_as_bytes, Offsets, Ptr};
pub(crate) use read::read;
pub(crate) use write::write;

pub(crate) fn size(light: &Light) -> u32 {
    let parent_size = (light.parent_indices.len() as u32) * 4;
    use mech3ax_types::AsBytes as _;
    LightRcC::SIZE.wrapping_add(parent_size)
}

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct LightRcC {
    recalc: i32,               // 000, 1
    field004: i32,             // 004, 1
    orientation: Vec3,         // 008 (Z = 0)
    translate: Vec3,           // 020 (0)
    euler_angles: Vec3,        // 032 (0)
    world_translate: Vec3,     // 044 (0)
    transform: AffineMatrix,   // 056 (0)
    world_point_source: Vec3,  // 104 (0)
    field116: Vec3,            // 116 (0)
    camera_point_source: Vec3, // 128 (0)
    world_view_vector: Vec3,   // 140 (0, 1.0, 0)
    camera_view_vector: Vec3,  // 152 (0)
    diffuse: f32,              // 164
    ambient: f32,              // 168
    color: Color,              // 172
    directional: i32,          // 184 (0)
    directed_source: i32,      // 188 (1)
    point_source: i32,         // 192 (0)
    saturated: i32,            // 196 (1)
    field200: i32,             // 200, 1
    range_near: f32,           // 204
    range_far: f32,            // 208
    range_far_sq: f32,         // 212
    range_inv: f32,            // 216
    parent_count: i32,         // 220
    parent_ptr: Ptr,           // 224
}
impl_as_bytes!(LightRcC, 228);

const WORLD_VIEW: Vec3 = Vec3 {
    x: 0.0,
    y: 1.0,
    z: 0.0,
};
