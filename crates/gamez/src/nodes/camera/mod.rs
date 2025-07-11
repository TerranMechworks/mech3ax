mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::{AffineMatrix, IndexO32, Vec3};
use mech3ax_types::{impl_as_bytes, AsBytes as _, Offsets};
pub(crate) use read::read;
pub(crate) use write::write;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct CameraC {
    world_index: IndexO32,             // 000
    window_index: IndexO32,            // 004
    focus_node_xy: IndexO32,           // 008
    focus_node_xz: IndexO32,           // 012
    flags: u32,                        // 016
    translate: Vec3,                   // 020
    rotate: Vec3,                      // 032
    world_translate: Vec3,             // 044
    world_rotate: Vec3,                // 056
    ancestor_matrix: AffineMatrix,     // 068
    view_vector: Vec3,                 // 116
    translate_matrix: AffineMatrix,    // 128
    clip_near: f32,                    // 176
    clip_far: f32,                     // 180
    clip_near_in_world: Vec3,          // 184
    clip_far_in_world: Vec3,           // 196
    lod_multiplier: f32,               // 208
    lod_mul_inv_sq: f32,               // 212
    fov_h_scale: f32,                  // 216
    fov_v_scale: f32,                  // 220
    fov_h_base: f32,                   // 224
    fov_v_base: f32,                   // 228
    fov_h_scaled: f32,                 // 232
    fov_v_scaled: f32,                 // 236
    fov_h_half: f32,                   // 240
    fov_v_half: f32,                   // 244
    clip_recalc: i32,                  // 248
    clip_unknown: Vec3,                // 252
    clip_far_plane_bottom_left: Vec3,  // 264
    clip_far_plane_bottom_right: Vec3, // 276
    clip_far_plane_top_left: Vec3,     // 288
    clip_far_plane_top_right: Vec3,    // 300
    fov_recalc: i32,                   // 312
    fov_right_dir_local: Vec3,         // 316
    fov_left_dir_local: Vec3,          // 328
    fov_bottom_dir_local: Vec3,        // 340
    fov_top_dir_local: Vec3,           // 352
    fov_front_dir_local: Vec3,         // 364
    fov_back_dir_local: Vec3,          // 376
    rotate_recalc: i32,                // 388
    fov_right_dir_world: Vec3,         // 392
    fov_left_dir_world: Vec3,          // 404
    fov_bottom_dir_world: Vec3,        // 416
    fov_top_dir_world: Vec3,           // 428
    fov_front_dir_world: Vec3,         // 440
    fov_back_dir_world: Vec3,          // 452
    field464: i32,                     // 464
    fov_h_cot: f32,                    // 468
    fov_v_cot: f32,                    // 472
    field476: i32,                     // 476
    zone_set_valid: i32,               // 480
    zone_set: u32,                     // 484
}
impl_as_bytes!(CameraC, 488);

fn cotangent(value: f32) -> f32 {
    // must perform this calculation with doubles to avoid loss of precision
    (1.0f64 / (value as f64).tan()) as f32
}

pub(crate) const fn size() -> u32 {
    CameraC::SIZE
}
