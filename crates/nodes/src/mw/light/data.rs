use super::info::LIGHT_NAME;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::nodes::mw::Light;
use mech3ax_api_types::{Color, Range, Vec3};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::light::LightFlagsU32 as LightFlags;
use mech3ax_common::{Result, assert_that};
use mech3ax_types::{AsBytes as _, Maybe, Zeros, impl_as_bytes};
use std::io::{Read, Write};

type Flags = Maybe<u32, LightFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct LightMwC {
    direction: Vec3,     // 000
    translation: Vec3,   // 012
    zero024: Zeros<112>, // 024
    one136: f32,         // 136
    zero140: f32,        // 140
    zero144: f32,        // 144
    zero148: f32,        // 148
    zero152: f32,        // 152
    diffuse: f32,        // 156
    ambient: f32,        // 160
    color: Color,        // 164
    flags: Flags,        // 176
    range: Range,        // 180
    range_near_sq: f32,  // 188
    range_far_sq: f32,   // 192
    range_inv: f32,      // 196
    parent_count: u32,   // 200
    parent_ptr: u32,     // 204
}
impl_as_bytes!(LightMwC, 208);

fn assert_light(light: &LightMwC, offset: usize) -> Result<()> {
    assert_that!(
        "light translation",
        light.translation == Vec3::DEFAULT,
        offset + 12
    )?;
    assert_that!("light field 024", zero light.zero024, offset + 24)?;

    assert_that!("light field 136", light.one136 == 1.0, offset + 136)?;
    assert_that!("light field 140", light.zero140 == 0.0, offset + 140)?;
    assert_that!("light field 144", light.zero144 == 0.0, offset + 144)?;
    assert_that!("light field 148", light.zero148 == 0.0, offset + 148)?;
    assert_that!("light field 152", light.zero152 == 0.0, offset + 152)?;

    assert_that!("light diffuse", 0.0 <= light.diffuse <= 1.0, offset + 156)?;
    assert_that!("light ambient", 0.0 <= light.ambient <= 1.0, offset + 160)?;

    assert_that!(
        "light color",
        light.color == Color::WHITE_NORM,
        offset + 164
    )?;

    let flags = assert_that!("light flags", flags  light.flags, offset + 176)?;
    assert_that!("light flag", flags == LightFlags::DEFAULT, offset + 176)?;

    assert_that!("light range near", light.range.min > 0.0, offset + 180)?;
    assert_that!(
        "light range far",
        light.range.max > light.range.min,
        offset + 184
    )?;
    let expected = light.range.min * light.range.min;
    assert_that!(
        "light range near sq",
        light.range_near_sq == expected,
        offset + 188
    )?;
    let expected = light.range.max * light.range.max;
    assert_that!(
        "light range far sq",
        light.range_far_sq == expected,
        offset + 192
    )?;
    let expected = 1.0 / (light.range.max - light.range.min);
    assert_that!("light range inv", light.range_inv == expected, offset + 196)?;

    assert_that!("light parent count", light.parent_count == 1, offset + 200)?;
    assert_that!("light parent ptr", light.parent_ptr != 0, offset + 204)?;
    Ok(())
}

pub(crate) fn read(read: &mut CountingReader<impl Read>, data_ptr: u32) -> Result<Light> {
    let light: LightMwC = read.read_struct()?;

    assert_light(&light, read.prev)?;

    // read as a result of parent_count, but is always 0 (= world node index)
    let light_parent = read.read_u32()?;
    assert_that!("light parent", light_parent == 0, read.prev)?;

    Ok(Light {
        name: LIGHT_NAME.to_owned(),
        direction: light.direction,
        diffuse: light.diffuse,
        ambient: light.ambient,
        color: light.color,
        range: light.range,
        parent_ptr: light.parent_ptr,
        data_ptr,
    })
}

pub(crate) fn write(write: &mut CountingWriter<impl Write>, light: &Light) -> Result<()> {
    let light = LightMwC {
        direction: light.direction,
        translation: Vec3::DEFAULT,
        zero024: Zeros::new(),
        one136: 1.0,
        zero140: 0.0,
        zero144: 0.0,
        zero148: 0.0,
        zero152: 0.0,
        diffuse: light.diffuse,
        ambient: light.ambient,
        color: light.color,
        flags: LightFlags::DEFAULT.maybe(),
        range: light.range,
        range_near_sq: light.range.min * light.range.min,
        range_far_sq: light.range.max * light.range.max,
        range_inv: 1.0 / (light.range.max - light.range.min),
        parent_count: 1,
        parent_ptr: light.parent_ptr,
    };
    write.write_struct(&light)?;
    // written as a result of parent_count, but is always 0 (= world node index)
    write.write_u32(0)?;
    Ok(())
}

pub(crate) fn size() -> u32 {
    LightMwC::SIZE + 4
}
