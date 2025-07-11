use super::{cotangent, CameraC};
use mech3ax_api_types::gamez::nodes::Camera;
use mech3ax_api_types::{AffineMatrix, Vec3};
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{chk, Result};
use std::io::Read;

pub(crate) fn read(read: &mut CountingReader<impl Read>) -> Result<Camera> {
    let camera: CameraC = read.read_struct()?;
    assert_camera(&camera, read.prev)
}

fn assert_camera(camera: &CameraC, offset: usize) -> Result<Camera> {
    // TODO: count validation
    let world_index = chk!(offset, ?camera.world_index)?;
    let window_index = chk!(offset, ?camera.window_index)?;
    let focus_node_xy = chk!(offset, ?camera.focus_node_xy)?;
    let focus_node_xz = chk!(offset, ?camera.focus_node_xz)?;
    chk!(offset, camera.flags == 0)?; // TODO
    chk!(offset, camera.translate == Vec3::DEFAULT)?;
    chk!(offset, camera.rotate == Vec3::DEFAULT)?;
    chk!(offset, camera.world_translate == Vec3::DEFAULT)?;
    chk!(offset, camera.world_rotate == Vec3::DEFAULT)?;
    chk!(offset, camera.ancestor_matrix == AffineMatrix::DEFAULT)?;
    chk!(offset, camera.view_vector == Vec3::DEFAULT)?;
    chk!(offset, camera.translate_matrix == AffineMatrix::DEFAULT)?;
    chk!(offset, camera.clip_near > 0.0)?;
    chk!(offset, camera.clip_far > camera.clip_near)?;
    chk!(offset, camera.clip_near_in_world == Vec3::DEFAULT)?;
    chk!(offset, camera.clip_far_in_world == Vec3::DEFAULT)?;
    chk!(offset, camera.lod_multiplier > 0.0)?;
    let lod_mul_inv_sq = 1.0 / (camera.lod_multiplier * camera.lod_multiplier);
    chk!(offset, camera.lod_mul_inv_sq == lod_mul_inv_sq)?;
    chk!(offset, camera.fov_h_scale > 0.0)?;
    chk!(offset, camera.fov_v_scale > 0.0)?;
    chk!(offset, camera.fov_h_base > 0.0)?;
    chk!(offset, camera.fov_v_base > 0.0)?;
    let fov_h_scaled = camera.fov_h_base * camera.fov_h_scale;
    chk!(offset, camera.fov_h_scaled == fov_h_scaled)?;
    let fov_v_scaled = camera.fov_v_base * camera.fov_v_scale;
    chk!(offset, camera.fov_v_scaled == fov_v_scaled)?;
    chk!(offset, camera.fov_h_half == fov_h_scaled / 2.0)?;
    chk!(offset, camera.fov_v_half == fov_v_scaled / 2.0)?;
    chk!(offset, camera.clip_recalc == 1)?;
    chk!(offset, camera.clip_unknown == Vec3::DEFAULT)?;
    chk!(offset, camera.clip_far_plane_bottom_left == Vec3::DEFAULT)?;
    chk!(offset, camera.clip_far_plane_bottom_right == Vec3::DEFAULT)?;
    chk!(offset, camera.clip_far_plane_top_left == Vec3::DEFAULT)?;
    chk!(offset, camera.clip_far_plane_top_right == Vec3::DEFAULT)?;
    chk!(offset, camera.fov_recalc == 1)?;
    chk!(offset, camera.fov_right_dir_local == Vec3::DEFAULT)?;
    chk!(offset, camera.fov_left_dir_local == Vec3::DEFAULT)?;
    chk!(offset, camera.fov_bottom_dir_local == Vec3::DEFAULT)?;
    chk!(offset, camera.fov_top_dir_local == Vec3::DEFAULT)?;
    chk!(offset, camera.fov_front_dir_local == Vec3::DEFAULT)?;
    chk!(offset, camera.fov_back_dir_local == Vec3::DEFAULT)?;
    chk!(offset, camera.rotate_recalc == 1)?;
    chk!(offset, camera.fov_right_dir_world == Vec3::DEFAULT)?;
    chk!(offset, camera.fov_left_dir_world == Vec3::DEFAULT)?;
    chk!(offset, camera.fov_bottom_dir_world == Vec3::DEFAULT)?;
    chk!(offset, camera.fov_top_dir_world == Vec3::DEFAULT)?;
    chk!(offset, camera.fov_front_dir_world == Vec3::DEFAULT)?;
    chk!(offset, camera.fov_back_dir_world == Vec3::DEFAULT)?;
    chk!(offset, camera.field464 == 0)?;
    let fov_h_cot = cotangent(camera.fov_h_half);
    chk!(offset, camera.fov_h_cot == fov_h_cot)?;
    let fov_v_cot = cotangent(camera.fov_v_half);
    chk!(offset, camera.fov_v_cot == fov_v_cot)?;
    chk!(offset, camera.field476 == 0)?;
    chk!(offset, camera.zone_set_valid == 0)?;
    chk!(offset, camera.zone_set == 0xFFFFFF00)?;

    Ok(Camera {
        world_index,
        window_index,
        focus_node_xy,
        focus_node_xz,
        clip_near: camera.clip_near,
        clip_far: camera.clip_far,
        lod_multiplier: camera.lod_multiplier,
        fov_h_scale: camera.fov_h_scale,
        fov_v_scale: camera.fov_v_scale,
        fov_h_base: camera.fov_h_base,
        fov_v_base: camera.fov_v_base,
    })
}
