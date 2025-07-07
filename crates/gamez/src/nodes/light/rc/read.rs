use super::{LightRcC, WORLD_VIEW};
use crate::nodes::check::node_count;
use crate::nodes::helpers::read_node_indices;
use mech3ax_api_types::gamez::nodes::Light;
use mech3ax_api_types::{AffineMatrix, Range, Vec3};
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{chk, Result};
use mech3ax_types::Ptr;
use std::io::Read;

struct LightTemp {
    light: Light,
    parent_count: u16,
}

pub(crate) fn read(read: &mut CountingReader<impl Read>) -> Result<Light> {
    let light: LightRcC = read.read_struct()?;
    let LightTemp {
        mut light,
        parent_count,
    } = assert_light(&light, read.prev)?;

    // read as a result of parent_count, which is always 1 in a normal ZBD,
    // and should always be 0 (= world node index)
    light.parent_indices = read_node_indices!(read, parent_count, |idx, cnt| {
        format!("light node parent index {}/{}", idx, cnt)
    })?;

    Ok(light)
}

fn assert_light(light: &LightRcC, offset: usize) -> Result<LightTemp> {
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
    let parent_count = chk!(offset, node_count(light.parent_count))?;
    chk!(offset, light.parent_ptr != Ptr::NULL)?;

    let range = Range {
        min: light.range_near,
        max: light.range_far,
    };

    // TODO
    let licht = Light {
        recalc: light.recalc,
        field004: light.field004,
        orientation: light.orientation,
        translate: light.translate,
        diffuse: light.diffuse,
        ambient: light.ambient,
        color: light.color,
        directional: light.directional,
        directed_source: light.directed_source,
        point_source: light.point_source,
        saturated: light.saturated,
        field200: light.field200,
        range,
        parent_indices: Vec::new(),
        parent_ptr: light.parent_ptr.0,
    };

    Ok(LightTemp {
        light: licht,
        parent_count,
    })
}
