use super::{LightRcC, WORLD_VIEW};
use mech3ax_api_types::gamez::nodes::Light;
use mech3ax_api_types::{AffineMatrix, Vec3};
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{chk, Result};
use mech3ax_types::Ptr;
use std::io::Read;

pub(crate) fn read(read: &mut CountingReader<impl Read>) -> Result<Light> {
    let light: LightRcC = read.read_struct()?;
    assert_light(&light, read.prev)?;

    // TODO: read_node_indices?
    // read as a result of parent_count, but is always 0 (= world node index)
    let light_parent = read.read_i32()?;
    // chk!("light parent", light_parent == 0, read.prev)?;

    Ok(Light {})
}

fn assert_light(light: &LightRcC, offset: usize) -> Result<()> {
    chk!(offset, light.recalc == 1)?;
    chk!(offset, light.field004 == 1)?;
    // TODO (always Z = 0.0)
    chk!(offset, light.orientation.z == 0.0)?;
    // TODO (always 0)
    chk!(offset, light.translate == Vec3::DEFAULT)?;
    chk!(offset, light.euler_angles == Vec3::DEFAULT)?;
    chk!(offset, light.world_translate == Vec3::DEFAULT)?;
    chk!(offset, light.transform == AffineMatrix::DEFAULT)?;
    chk!(offset, light.world_point_source == Vec3::DEFAULT)?;
    chk!(offset, light.field116 == Vec3::DEFAULT)?;
    chk!(offset, light.camera_point_source == Vec3::DEFAULT)?;
    chk!(offset, light.world_view_vector == WORLD_VIEW)?;
    chk!(offset, light.camera_view_vector == Vec3::DEFAULT)?;
    // TODO
    // chk!(offset, light.diffuse == f32::DEFAULT)?;
    // chk!(offset, light.ambient == f32::DEFAULT)?;
    // chk!(offset, light.color == Color::DEFAULT)?;
    // chk!(offset, light.directional == i32::DEFAULT)?; 0
    // chk!(offset, light.directed_source == i32::DEFAULT)?;//  (1)
    // chk!(offset, light.point_source == i32::DEFAULT)?;//  (0)
    // chk!(offset, light.saturated == i32::DEFAULT)?;//  (1)
    chk!(offset, light.field200 == 1)?;
    chk!(offset, light.range_near > 0.0)?;
    chk!(offset, light.range_far > light.range_near)?;
    let range_far_sq = light.range_far * light.range_far;
    chk!(offset, light.range_far_sq == range_far_sq)?;
    let range_inv = 1.0 / (light.range_far - light.range_near);
    chk!(offset, light.range_inv == range_inv)?;
    chk!(offset, light.parent_count == 1)?;
    chk!(offset, light.parent_ptr != Ptr::NULL)?;

    Ok(())
}
