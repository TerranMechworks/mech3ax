use super::{LightRcC, WORLD_VIEW};
use crate::nodes::check::color;
use crate::nodes::helpers::read_node_indices;
use mech3ax_api_types::gamez::nodes::{Light, LightFlags};
use mech3ax_api_types::{AffineMatrix, Color, Count, Range, Vec3};
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{chk, Result};
use mech3ax_types::Ptr;
use std::io::Read;

struct LightTemp {
    light: Light,
    parent_count: Count,
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

fn float(value: f32) -> Result<f32, String> {
    if value < 0.0 || value > 1.0 {
        Err(format!("expected {} in 0.0..=1.0", value))
    } else {
        Ok(value)
    }
}

fn assert_light(light: &LightRcC, offset: usize) -> Result<LightTemp> {
    let recalc = chk!(offset, ?light.recalc)?;
    let unk1 = chk!(offset, ?light.field004)?;

    chk!(offset, light.euler_angles == Vec3::DEFAULT)?;
    chk!(offset, light.world_translate == Vec3::DEFAULT)?;
    chk!(offset, light.transform == AffineMatrix::DEFAULT)?;
    chk!(offset, light.world_point_source == Vec3::DEFAULT)?;
    chk!(offset, light.field116 == Vec3::DEFAULT)?;
    chk!(offset, light.camera_point_source == Vec3::DEFAULT)?;
    chk!(offset, light.world_view_vector == WORLD_VIEW)?;
    chk!(offset, light.camera_view_vector == Vec3::DEFAULT)?;

    let diffuse = chk!(offset, float(light.diffuse))?;
    let ambient = chk!(offset, float(light.ambient))?;
    chk!(offset, color(light.color.r))?;
    chk!(offset, color(light.color.g))?;
    chk!(offset, color(light.color.b))?;

    let directional = chk!(offset, ?light.directional)?;
    let directed_source = chk!(offset, ?light.directed_source)?;
    let point_source = chk!(offset, ?light.point_source)?;
    let saturated = chk!(offset, ?light.saturated)?;
    let unk9 = chk!(offset, ?light.field200)?;

    chk!(offset, light.range_near > 0.0)?;
    chk!(offset, light.range_far > light.range_near)?;

    let range_far_sq = light.range_far * light.range_far;
    chk!(offset, light.range_far_sq == range_far_sq)?;
    let range_inv = 1.0 / (light.range_far - light.range_near);
    chk!(offset, light.range_inv == range_inv)?;

    let parent_count = chk!(offset, ?light.parent_count)?;
    chk!(offset, light.parent_ptr != Ptr::NULL)?;

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
    if unk9 {
        flags |= LightFlags::UNK9;
    }

    let range = Range {
        min: light.range_near,
        max: light.range_far,
    };

    let licht = Light {
        flags,
        orientation: light.orientation,
        translate: light.translate,
        diffuse,
        ambient,
        color: light.color,
        color_ambient: Color::BLACK,
        color_diffuse_mixed: Color::BLACK,
        color_ambient_mixed: Color::BLACK,
        color_da_combined: Color::BLACK,
        range,
        parent_indices: Vec::new(),
        parent_ptr: light.parent_ptr.0,
    };

    Ok(LightTemp {
        light: licht,
        parent_count,
    })
}
