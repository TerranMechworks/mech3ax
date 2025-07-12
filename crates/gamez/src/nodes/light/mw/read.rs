use super::{LightFlags as LFlags, LightMwC, WORLD_VIEW};
use crate::nodes::check::{color, ptr};
use crate::nodes::helpers::read_node_indices;
use mech3ax_api_types::gamez::nodes::{Light, LightFlags};
use mech3ax_api_types::{AffineMatrix, Color, Count, Range, Vec3};
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{Result, chk};
use std::io::Read;

struct LightTemp {
    light: Light,
    parent_count: Count,
}

pub(crate) fn read(read: &mut CountingReader<impl Read>) -> Result<Light> {
    let light: LightMwC = read.read_struct()?;
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

fn assert_light(light: &LightMwC, offset: usize) -> Result<LightTemp> {
    let bitflags = chk!(offset, ?light.flags)?;

    chk!(offset, light.euler_angles == Vec3::DEFAULT)?;
    chk!(offset, light.world_translate == Vec3::DEFAULT)?;
    chk!(offset, light.transform == AffineMatrix::DEFAULT)?;
    chk!(offset, light.world_point_source == Vec3::DEFAULT)?;
    chk!(offset, light.field108 == Vec3::DEFAULT)?;
    chk!(offset, light.camera_point_source == Vec3::DEFAULT)?;
    chk!(offset, light.world_view_vector == WORLD_VIEW)?;
    chk!(offset, light.camera_view_vector == Vec3::DEFAULT)?;

    let diffuse = chk!(offset, float(light.diffuse))?;
    let ambient = chk!(offset, float(light.ambient))?;
    chk!(offset, color(light.color.r))?;
    chk!(offset, color(light.color.g))?;
    chk!(offset, color(light.color.b))?;

    chk!(offset, light.range_near > 0.0)?;
    chk!(offset, light.range_far > light.range_near)?;

    let range_near_sq = light.range_near * light.range_near;
    chk!(offset, light.range_near_sq == range_near_sq)?;
    let range_far_sq = light.range_far * light.range_far;
    chk!(offset, light.range_far_sq == range_far_sq)?;
    let range_inv = 1.0 / (light.range_far - light.range_near);
    chk!(offset, light.range_inv == range_inv)?;

    let parent_count = chk!(offset, ?light.parent_count)?;
    chk!(offset, ptr(light.parent_ptr, parent_count))?;

    let mut flags = LightFlags::empty();
    if bitflags.contains(LFlags::RECALC) {
        flags |= LightFlags::RECALC;
    }
    if bitflags.contains(LFlags::UNK1) {
        flags |= LightFlags::UNK1;
    }
    if bitflags.contains(LFlags::DIRECTIONAL) {
        flags |= LightFlags::DIRECTIONAL;
    }
    if bitflags.contains(LFlags::DIRECTED_SOURCE) {
        flags |= LightFlags::DIRECTED_SOURCE;
    }
    if bitflags.contains(LFlags::POINT_SOURCE) {
        flags |= LightFlags::POINT_SOURCE;
    }
    if bitflags.contains(LFlags::SATURATED) {
        flags |= LightFlags::SATURATED;
    }
    if bitflags.contains(LFlags::SUBDIVIDE) {
        flags |= LightFlags::SUBDIVIDE;
    }
    if bitflags.contains(LFlags::STATIC) {
        flags |= LightFlags::STATIC;
    }
    if bitflags.contains(LFlags::COLOR) {
        flags |= LightFlags::COLOR;
    }
    if bitflags.contains(LFlags::UNK9) {
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
