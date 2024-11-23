use super::info::LIGHT_NAME;
use bytemuck::{AnyBitPattern, NoUninit};
use log::{debug, trace};
use mech3ax_api_types::nodes::rc::Light;
use mech3ax_api_types::{Color, Range};
use mech3ax_common::assert::assert_all_zero;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::Zeros;
use mech3ax_types::{impl_as_bytes, AsBytes as _};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct LightRcC {
    unk000: u32,         // 000
    unk004: u32,         // 004
    unk008: f32,         // 008
    unk012: f32,         // 012
    zero016: Zeros<128>, // 016
    unk144: f32,         // 144
    unk148: u32,         // 148
    unk152: u32,         // 152
    unk156: u32,         // 156
    unk160: u32,         // 160
    color: Color,        // 164
    unk176: u32,         // 176
    unk180: u32,         // 180
    unk184: u32,         // 184
    unk188: u32,         // 188
    unk192: u32,         // 192
    unk196: u32,         // 196
    unk200: u32,         // 200
    range: Range,        // 204
    range_far_sq: f32,   // 212
    range_inv: f32,      // 216
    parent_count: u32,   // 220
    parent_ptr: u32,     // 224
}
impl_as_bytes!(LightRcC, 228);

fn assert_light(light: &LightRcC, offset: usize) -> Result<()> {
    assert_that!("light field 000", light.unk000 == 1, offset + 0)?;
    assert_that!("light field 004", light.unk004 == 1, offset + 4)?;
    // unk008
    // unk012
    assert_all_zero("light field 016", offset + 16, &light.zero016.0)?;
    assert_that!("light field 144", light.unk144 == 1.0, offset + 144)?;
    assert_that!("light field 148", light.unk148 == 0, offset + 148)?;
    assert_that!("light field 152", light.unk152 == 0, offset + 152)?;
    assert_that!("light field 156", light.unk156 == 0, offset + 156)?;
    assert_that!("light field 160", light.unk160 == 0, offset + 160)?;
    assert_that!("light color r", 0.0 <= light.color.r <= 1.0, offset + 164)?;
    assert_that!("light color g", 0.0 <= light.color.g <= 1.0, offset + 168)?;
    assert_that!("light color b", 0.0 <= light.color.b <= 1.0, offset + 172)?;
    assert_that!("light field 176", light.unk176 == 0, offset + 176)?;
    assert_that!("light field 180", light.unk180 == 0, offset + 180)?;
    assert_that!("light field 184", light.unk184 == 0, offset + 184)?;
    assert_that!("light field 188", light.unk188 == 1, offset + 188)?;
    assert_that!("light field 192", light.unk192 == 0, offset + 192)?;
    assert_that!("light field 196", light.unk196 == 1, offset + 196)?;
    assert_that!("light field 200", light.unk200 == 1, offset + 200)?;
    assert_that!("light range near", light.range.min > 0.0, offset + 204)?;
    assert_that!(
        "light range far",
        light.range.max > light.range.min,
        offset + 208
    )?;
    let expected = light.range.max * light.range.max;
    assert_that!(
        "light range far sq",
        light.range_far_sq == expected,
        offset + 212
    )?;
    let expected = 1.0 / (light.range.max - light.range.min);
    assert_that!("light range inv", light.range_inv == expected, offset + 216)?;
    assert_that!("light parent count", light.parent_count == 1, offset + 220)?;
    assert_that!("light parent ptr", light.parent_ptr != 0, offset + 224)?;
    Ok(())
}

pub fn read(read: &mut CountingReader<impl Read>, data_ptr: u32, index: usize) -> Result<Light> {
    debug!(
        "Reading light node data {} (rc, {}) at {}",
        index,
        LightRcC::SIZE,
        read.offset
    );
    let light: LightRcC = read.read_struct()?;
    trace!("{:#?}", light);

    assert_light(&light, read.prev)?;

    // read as a result of parent_count, but is always 0 (= world node index)
    let light_parent = read.read_u32()?;
    assert_that!("light parent", light_parent == 0, read.prev)?;

    Ok(Light {
        name: LIGHT_NAME.to_owned(),
        unk008: light.unk008,
        unk012: light.unk012,
        // direction: light.direction,
        // diffuse: light.diffuse,
        // ambient: light.ambient,
        color: light.color,
        range: light.range,
        parent_ptr: light.parent_ptr,
        data_ptr,
    })
}

pub fn write(write: &mut CountingWriter<impl Write>, light: &Light, index: usize) -> Result<()> {
    debug!(
        "Writing light node data {} (rc, {}) at {}",
        index,
        LightRcC::SIZE,
        write.offset
    );
    let light = LightRcC {
        unk000: 1,
        unk004: 1,
        unk008: light.unk008,
        unk012: light.unk012,
        zero016: Zeros::new(),
        unk144: 1.0,
        unk148: 0,
        unk152: 0,
        unk156: 0,
        unk160: 0,
        color: light.color,
        unk176: 0,
        unk180: 0,
        unk184: 0,
        unk188: 1,
        unk192: 0,
        unk196: 1,
        unk200: 1,
        range: light.range,
        range_far_sq: light.range.max * light.range.max,
        range_inv: 1.0 / (light.range.max - light.range.min),
        parent_count: 1,
        parent_ptr: light.parent_ptr,
    };
    trace!("{:#?}", light);
    write.write_struct(&light)?;
    // written as a result of parent_count, but is always 0 (= world node index)
    write.write_u32(0)?;
    Ok(())
}

pub fn size() -> u32 {
    LightRcC::SIZE + 4
}
