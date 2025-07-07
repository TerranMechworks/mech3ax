use super::{LightRcC, WORLD_VIEW};
use crate::nodes::helpers::write_node_indices;
use mech3ax_api_types::gamez::nodes::Light;
use mech3ax_api_types::{AffineMatrix, Vec3};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{assert_len, err, Result};
use mech3ax_types::Ptr;
use std::io::Write;

pub(crate) fn write(write: &mut CountingWriter<impl Write>, light: &Light) -> Result<()> {
    let parent_count = assert_len!(i32, light.parent_indices.len(), "light parent indices")?;

    let range_near = light.range.min;
    let range_far = light.range.max;
    let range_far_sq = range_far * range_far;
    let range_diff = range_far - range_near;
    if range_diff == 0.0 {
        return Err(err!("range near equals range far"));
    }
    let range_inv = 1.0 / range_diff;

    let licht = LightRcC {
        recalc: light.recalc,
        field004: light.field004,
        orientation: light.orientation,
        translate: light.translate,
        euler_angles: Vec3::DEFAULT,
        world_translate: Vec3::DEFAULT,
        transform: AffineMatrix::DEFAULT,
        world_point_source: Vec3::DEFAULT,
        field116: Vec3::DEFAULT,
        camera_point_source: Vec3::DEFAULT,
        world_view_vector: WORLD_VIEW,
        camera_view_vector: Vec3::DEFAULT,
        diffuse: light.diffuse,
        ambient: light.ambient,
        color: light.color,
        directional: light.directional,
        directed_source: light.directed_source,
        point_source: light.point_source,
        saturated: light.saturated,
        field200: light.field200,
        range_near,
        range_far,
        range_far_sq,
        range_inv,
        parent_count,
        parent_ptr: Ptr(light.parent_ptr),
    };
    write.write_struct(&licht)?;

    write_node_indices(write, &light.parent_indices)?;
    Ok(())
}
