use super::CameraC;
use crate::nodes::math::cotangent;
use mech3ax_api_types::gamez::nodes::Camera;
use mech3ax_api_types::{AffineMatrix, Vec3};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::Result;
use std::io::Write;

pub(crate) fn write(write: &mut CountingWriter<impl Write>, camera: &Camera) -> Result<()> {
    let lod_mul_inv_sq = 1.0 / (camera.lod_multiplier * camera.lod_multiplier);
    let fov_h_scaled = camera.fov_h_base * camera.fov_h_scale;
    let fov_v_scaled = camera.fov_v_base * camera.fov_v_scale;
    let fov_h_half = fov_h_scaled / 2.0;
    let fov_v_half = fov_v_scaled / 2.0;
    let fov_h_cot = cotangent(fov_h_half);
    let fov_v_cot = cotangent(fov_v_half);

    let camera = CameraC {
        world_index: camera.world_index.map(Into::into).unwrap_or(-1),
        window_index: camera.window_index.map(Into::into).unwrap_or(-1),
        focus_node_xy: camera.focus_node_xy.map(Into::into).unwrap_or(-1),
        focus_node_xz: camera.focus_node_xz.map(Into::into).unwrap_or(-1),
        flags: 0,
        translate: Vec3::DEFAULT,
        rotate: Vec3::DEFAULT,
        world_translate: Vec3::DEFAULT,
        world_rotate: Vec3::DEFAULT,
        ancestor_matrix: AffineMatrix::DEFAULT,
        view_vector: Vec3::DEFAULT,
        translate_matrix: AffineMatrix::DEFAULT,
        clip_near: camera.clip_near,
        clip_far: camera.clip_far,
        clip_near_in_world: Vec3::DEFAULT,
        clip_far_in_world: Vec3::DEFAULT,
        lod_multiplier: camera.lod_multiplier,
        lod_mul_inv_sq,
        fov_h_scale: camera.fov_h_scale,
        fov_v_scale: camera.fov_v_scale,
        fov_h_base: camera.fov_h_base,
        fov_v_base: camera.fov_v_base,
        fov_h_scaled,
        fov_v_scaled,
        fov_h_half,
        fov_v_half,
        clip_recalc: 1,
        clip_unknown: Vec3::DEFAULT,
        clip_far_plane_bottom_left: Vec3::DEFAULT,
        clip_far_plane_bottom_right: Vec3::DEFAULT,
        clip_far_plane_top_left: Vec3::DEFAULT,
        clip_far_plane_top_right: Vec3::DEFAULT,
        fov_recalc: 1,
        fov_right_dir_local: Vec3::DEFAULT,
        fov_left_dir_local: Vec3::DEFAULT,
        fov_bottom_dir_local: Vec3::DEFAULT,
        fov_top_dir_local: Vec3::DEFAULT,
        fov_front_dir_local: Vec3::DEFAULT,
        fov_back_dir_local: Vec3::DEFAULT,
        rotate_recalc: 1,
        fov_right_dir_world: Vec3::DEFAULT,
        fov_left_dir_world: Vec3::DEFAULT,
        fov_bottom_dir_world: Vec3::DEFAULT,
        fov_top_dir_world: Vec3::DEFAULT,
        fov_front_dir_world: Vec3::DEFAULT,
        fov_back_dir_world: Vec3::DEFAULT,
        field464: 0,
        fov_h_cot,
        fov_v_cot,
        field476: 0,
        zone_set_valid: 0,
        zone_set: 0xFFFFFF00,
    };
    write.write_struct(&camera)?;
    Ok(())
}
