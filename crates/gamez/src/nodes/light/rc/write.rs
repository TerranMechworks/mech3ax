use super::{LightRcC, WORLD_VIEW};
use crate::nodes::helpers::write_node_indices;
use mech3ax_api_types::gamez::nodes::{Light, LightFlagsExhaustive};
use mech3ax_api_types::{AffineMatrix, Vec3};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{err, len, Result};
use mech3ax_types::{Ptr, SupportsMaybe as _};
use std::io::Write;

pub(crate) fn write(write: &mut CountingWriter<impl Write>, light: &Light) -> Result<()> {
    let parent_count = len!(light.parent_indices.len(), "light parent indices")?;

    let LightFlagsExhaustive {
        recalc,
        unk1,
        directional,
        directed_source,
        point_source,
        saturated,
        subdivide: _,
        static_: _,
        color: _,
        unk9,
        light_map: _,
        bicolored: _,
    } = light.flags.exhaustive();

    let range_near = light.range.min;
    let range_far = light.range.max;
    let range_far_sq = range_far * range_far;
    let range_diff = range_far - range_near;
    if range_diff == 0.0 {
        return Err(err!("range near equals range far"));
    }
    let range_inv = 1.0 / range_diff;

    let licht = LightRcC {
        recalc: recalc.maybe(),
        field004: unk1.maybe(),
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
        directional: directional.maybe(),
        directed_source: directed_source.maybe(),
        point_source: point_source.maybe(),
        saturated: saturated.maybe(),
        field200: unk9.maybe(),
        range_near,
        range_far,
        range_far_sq,
        range_inv,
        parent_count: parent_count.maybe(),
        parent_ptr: Ptr(light.parent_ptr),
    };
    write.write_struct(&licht)?;

    write_node_indices(write, &light.parent_indices)?;
    Ok(())
}
