use super::{LightFlags, LightPmC, WORLD_VIEW};
use crate::nodes::helpers::write_node_indices;
use mech3ax_api_types::gamez::nodes::{Light, LightFlagsExhaustive};
use mech3ax_api_types::{AffineMatrix, Vec3};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{err, len, Result};
use mech3ax_types::Ptr;
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
        subdivide,
        static_,
        color,
        unk9,
        light_map,
        bicolored,
    } = light.flags.exhaustive();

    let mut flags = LightFlags::empty();
    if recalc {
        flags |= LightFlags::RECALC;
    }
    if unk1 {
        flags |= LightFlags::UNK1;
    }
    if directional {
        flags |= LightFlags::DIRECTIONAL;
    }
    if directed_source {
        flags |= LightFlags::DIRECTED_SOURCE;
    }
    if point_source {
        flags |= LightFlags::POINT_SOURCE;
    }
    if saturated {
        flags |= LightFlags::SATURATED;
    }
    if subdivide {
        flags |= LightFlags::SUBDIVIDE;
    }
    if light_map {
        flags |= LightFlags::LIGHT_MAP;
    }
    if static_ {
        flags |= LightFlags::STATIC;
    }
    if color {
        flags |= LightFlags::COLOR;
    }
    if bicolored {
        flags |= LightFlags::BICOLORED;
    }
    if unk9 {
        flags |= LightFlags::UNK9;
    }

    let range_near = light.range.min;
    let range_far = light.range.max;
    let range_near_sq = range_near * range_near;
    let range_far_sq = range_far * range_far;
    let range_diff = range_far - range_near;
    if range_diff == 0.0 {
        return Err(err!("range near equals range far"));
    }
    let range_inv = 1.0 / range_diff;

    let licht = LightPmC {
        orientation: light.orientation,
        translate: light.translate,
        euler_angles: Vec3::DEFAULT,
        world_translate: Vec3::DEFAULT,
        transform: AffineMatrix::DEFAULT,
        world_point_source: Vec3::DEFAULT,
        field108: Vec3::DEFAULT,
        camera_point_source: Vec3::DEFAULT,
        world_view_vector: WORLD_VIEW,
        camera_view_vector: Vec3::DEFAULT,
        diffuse: light.diffuse,
        ambient: light.ambient,
        color: light.color,
        color_ambient: light.color_ambient,
        color_diffuse_mixed: light.color_diffuse_mixed,
        color_ambient_mixed: light.color_ambient_mixed,
        color_da_combined: light.color_da_combined,
        flags: flags.maybe(),
        range_near,
        range_far,
        range_near_sq,
        range_far_sq,
        range_inv,
        parent_count: parent_count.maybe(),
        parent_ptr: Ptr(light.parent_ptr),
    };
    write.write_struct(&licht)?;

    write_node_indices(write, &light.parent_indices)?;
    Ok(())
}
