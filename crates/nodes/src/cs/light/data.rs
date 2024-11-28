use super::info::LIGHT_NAME;
use bytemuck::{AnyBitPattern, NoUninit};
use log::debug;
use mech3ax_api_types::nodes::cs::Light;
use mech3ax_api_types::Range;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{impl_as_bytes, AsBytes as _, Zeros};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct LightCsC {
    unk000: f32,         // 000
    unk004: f32,         // 004
    zero008: Zeros<128>, // 008
    unk136: f32,         // 136
    zero140: u32,        // 140
    zero144: u32,        // 144
    zero148: u32,        // 148
    zero152: u32,        // 152
    unk156: f32,         // 156
    unk160: f32,         // 160
    unk164: f32,         // 164
    unk168: f32,         // 168
    unk172: f32,         // 172
    unk176: f32,         // 176
    unk180: f32,         // 180
    unk184: f32,         // 184
    unk188: f32,         // 188
    unk192: f32,         // 192
    unk196: f32,         // 196
    unk200: f32,         // 200
    unk204: f32,         // 204
    unk208: f32,         // 208
    unk212: f32,         // 212
    unk216: f32,         // 216
    unk220: f32,         // 220
    unk224: u32,         // 224
    range: Range,        // 228
    unk236: f32,         // 236
    range_far_sq: f32,   // 240
    range_inv: f32,      // 244
    parent_count: u32,   // 248
    parent_ptr: u32,     // 252
}
impl_as_bytes!(LightCsC, 256);

fn assert_light(light: &LightCsC, offset: usize) -> Result<()> {
    assert_that!("light field 000", light.unk000 == 0.0, offset + 0)?;
    assert_that!("light field 004", light.unk004 == 0.0, offset + 4)?;
    assert_that!("light field 008", zero light.zero008, offset + 8)?;
    assert_that!("light field 136", light.unk136 == 1.0, offset + 136)?;
    assert_that!("light field 140", light.zero140 == 0, offset + 140)?;
    assert_that!("light field 144", light.zero144 == 0, offset + 144)?;
    assert_that!("light field 148", light.zero148 == 0, offset + 148)?;
    assert_that!("light field 152", light.zero152 == 0, offset + 152)?;

    assert_that!("light field 156", light.unk156 == 0.0, offset + 156)?;
    assert_that!("light field 160", light.unk160 == 1.0, offset + 160)?;

    assert_that!("light field 164", light.unk164 == 1.0, offset + 164)?;
    assert_that!("light field 168", light.unk168 == 1.0, offset + 168)?;
    assert_that!("light field 172", light.unk172 == 1.0, offset + 172)?;
    assert_that!("light field 176", light.unk176 == 1.0, offset + 176)?;
    assert_that!("light field 180", light.unk180 == 1.0, offset + 180)?;
    assert_that!("light field 184", light.unk184 == 1.0, offset + 184)?;
    assert_that!("light field 188", light.unk188 == 0.0, offset + 188)?;
    assert_that!("light field 192", light.unk192 == 0.0, offset + 192)?;
    assert_that!("light field 196", light.unk196 == 0.0, offset + 196)?;
    assert_that!("light field 200", light.unk200 == 1.0, offset + 200)?;
    assert_that!("light field 204", light.unk204 == 1.0, offset + 204)?;
    assert_that!("light field 208", light.unk208 == 1.0, offset + 208)?;
    assert_that!("light field 212", light.unk212 == 1.0, offset + 212)?;
    assert_that!("light field 216", light.unk216 == 1.0, offset + 216)?;
    assert_that!("light field 220", light.unk220 == 1.0, offset + 220)?;

    assert_that!("light field 224", light.unk224 == 2225, offset + 224)?;

    assert_that!("light range near", light.range.min > 0.0, offset + 228)?;
    assert_that!(
        "light range far",
        light.range.max > light.range.min,
        offset + 232
    )?;
    assert_that!("light field 236", light.unk236 == 1024.0, offset + 236)?;
    let expected = light.range.max * light.range.max;
    assert_that!(
        "light range far sq",
        light.range_far_sq == expected,
        offset + 240
    )?;
    let expected = 1.0 / (light.range.max - light.range.min);
    assert_that!("light range inv", light.range_inv == expected, offset + 244)?;
    assert_that!("light parent count", light.parent_count == 1, offset + 248)?;
    assert_that!("light parent ptr", light.parent_ptr != 0, offset + 252)?;
    Ok(())
}

pub fn read(
    read: &mut CountingReader<impl Read>,
    data_ptr: u32,
    node_index: u32,
    index: usize,
) -> Result<Light> {
    debug!(
        "Reading light node data {} (cs, {}) at {}",
        index,
        LightCsC::SIZE,
        read.offset
    );
    let light: LightCsC = read.read_struct()?;

    assert_light(&light, read.prev)?;

    // read as a result of parent_count, but is always 0 (= world node index)
    let light_parent = read.read_u32()?;
    assert_that!("light parent", light_parent == 0, read.prev)?;

    Ok(Light {
        name: LIGHT_NAME.to_owned(),
        range: light.range,
        parent_ptr: light.parent_ptr,
        data_ptr,
        node_index,
    })
}

pub fn write(write: &mut CountingWriter<impl Write>, light: &Light, index: usize) -> Result<()> {
    debug!(
        "Writing light node data {} (cs, {}) at {}",
        index,
        LightCsC::SIZE,
        write.offset
    );

    let light = LightCsC {
        unk000: 0.0,
        unk004: 0.0,
        zero008: Zeros::new(),
        unk136: 1.0,
        zero140: 0,
        zero144: 0,
        zero148: 0,
        zero152: 0,
        unk156: 0.0,
        unk160: 1.0,
        unk164: 1.0,
        unk168: 1.0,
        unk172: 1.0,
        unk176: 1.0,
        unk180: 1.0,
        unk184: 1.0,
        unk188: 0.0,
        unk192: 0.0,
        unk196: 0.0,
        unk200: 1.0,
        unk204: 1.0,
        unk208: 1.0,
        unk212: 1.0,
        unk216: 1.0,
        unk220: 1.0,
        unk224: 2225,
        range: light.range,
        unk236: 1024.0,
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
