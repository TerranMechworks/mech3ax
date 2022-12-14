use super::node::{NodeVariantPm, NodeVariantsPm};
use crate::flags::NodeBitFlags;
use crate::types::ZONE_DEFAULT;
use log::{debug, trace};
use mech3ax_api_types::nodes::pm::Light;
use mech3ax_api_types::nodes::BoundingBox;
use mech3ax_api_types::{static_assert_size, Range, ReprSize as _, Vec3};
use mech3ax_common::assert::assert_all_zero;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_debug::Zeros;
use std::io::{Read, Write};

#[derive(Debug)]
#[repr(C)]
struct LightPmC {
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
static_assert_size!(LightPmC, 256);

const BBOX_LIGHT: BoundingBox = BoundingBox {
    a: Vec3 {
        x: 1.0,
        y: 1.0,
        z: -2.0,
    },
    b: Vec3 {
        x: 2.0,
        y: 2.0,
        z: -1.0,
    },
};
const LIGHT_NAME: &str = "sunlight";

pub fn assert_variants(node: NodeVariantsPm, offset: u32) -> Result<NodeVariantPm> {
    assert_that!("light name", &node.name == LIGHT_NAME, offset + 0)?;
    assert_that!(
        "light flags",
        node.flags == NodeBitFlags::DEFAULT | NodeBitFlags::UNK08,
        offset + 36
    )?;
    // zero040 (40) already asserted
    assert_that!("light field 044", node.unk044 == 0, offset + 44)?;
    assert_that!("light zone id", node.zone_id == ZONE_DEFAULT, offset + 48)?;
    // node_type (52) already asserted
    assert_that!("light data ptr", node.data_ptr != 0, offset + 56)?;
    assert_that!("light mesh index", node.mesh_index == -1, offset + 60)?;
    // environment_data (64) already asserted
    // action_priority (68) already asserted
    // action_callback (72) already asserted
    assert_that!(
        "light area partition",
        node.area_partition == None,
        offset + 76
    )?;
    assert_that!("light has parent", node.has_parent == false, offset + 84)?;
    // parent_array_ptr (88) already asserted
    assert_that!(
        "light children count",
        node.children_count == 0,
        offset + 92
    )?;
    // children_array_ptr (92) already asserted
    // zero096 (96) already asserted
    // zero100 (100) already asserted
    // zero104 (104) already asserted
    // zero108 (108) already asserted
    assert_that!("light field 112", node.unk112 == 0, offset + 112)?;
    assert_that!("light bbox 1", node.unk116 == BBOX_LIGHT, offset + 116)?;
    assert_that!(
        "light bbox 2",
        node.unk140 == BoundingBox::EMPTY,
        offset + 140
    )?;
    assert_that!(
        "light bbox 3",
        node.unk164 == BoundingBox::EMPTY,
        offset + 164
    )?;
    // zero188 (188) already asserted
    // zero192 (192) already asserted
    assert_that!("light field 196", node.unk196 == 0, offset + 196)?;
    // zero200 (200) already asserted
    // zero204 (204) already asserted
    Ok(NodeVariantPm::Light {
        data_ptr: node.data_ptr,
    })
}

pub fn make_variants(light: &Light) -> NodeVariantsPm {
    NodeVariantsPm {
        name: LIGHT_NAME.to_owned(),
        flags: NodeBitFlags::DEFAULT | NodeBitFlags::UNK08,
        unk044: 0,
        zone_id: ZONE_DEFAULT,
        data_ptr: light.data_ptr,
        mesh_index: -1,
        area_partition: None,
        has_parent: false,
        parent_array_ptr: 0,
        children_count: 0,
        children_array_ptr: 0,
        unk112: 0,
        unk116: BBOX_LIGHT,
        unk140: BoundingBox::EMPTY,
        unk164: BoundingBox::EMPTY,
        unk196: 0,
    }
}

fn assert_light(light: &LightPmC, offset: u32) -> Result<()> {
    assert_that!("light field 000", light.unk000 == -0.5235988, offset + 0)?;
    // unk008
    assert_all_zero("light field 008", offset + 8, &light.zero008.0)?;
    assert_that!("light field 136", light.unk136 == 1.0, offset + 136)?;
    assert_that!("light field 140", light.zero140 == 0, offset + 140)?;
    assert_that!("light field 144", light.zero144 == 0, offset + 144)?;
    assert_that!("light field 148", light.zero148 == 0, offset + 148)?;
    assert_that!("light field 152", light.zero152 == 0, offset + 152)?;

    let unk_combined = light.unk156 + light.unk160;

    assert_that!("light field 164", light.unk164 == 1.0, offset + 164)?;
    assert_that!("light field 168", light.unk168 == 1.0, offset + 168)?;
    assert_that!("light field 172", light.unk172 == 1.0, offset + 172)?;
    assert_that!("light field 176", light.unk176 == 1.0, offset + 176)?;
    assert_that!("light field 180", light.unk180 == 1.0, offset + 180)?;
    assert_that!("light field 184", light.unk184 == 1.0, offset + 184)?;
    assert_that!(
        "light field 188",
        light.unk188 == light.unk156,
        offset + 188
    )?;
    assert_that!(
        "light field 192",
        light.unk192 == light.unk156,
        offset + 192
    )?;
    assert_that!(
        "light field 196",
        light.unk196 == light.unk156,
        offset + 196
    )?;
    assert_that!(
        "light field 200",
        light.unk200 == light.unk160,
        offset + 200
    )?;
    assert_that!(
        "light field 204",
        light.unk204 == light.unk160,
        offset + 204
    )?;
    assert_that!(
        "light field 208",
        light.unk208 == light.unk160,
        offset + 208
    )?;
    assert_that!(
        "light field 212",
        light.unk212 == unk_combined,
        offset + 212
    )?;
    assert_that!(
        "light field 216",
        light.unk216 == unk_combined,
        offset + 216
    )?;
    assert_that!(
        "light field 220",
        light.unk220 == unk_combined,
        offset + 220
    )?;

    assert_that!("light field 224", light.unk224 == 2443, offset + 224)?;

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

pub fn read(read: &mut CountingReader<impl Read>, data_ptr: u32, index: usize) -> Result<Light> {
    debug!(
        "Reading light node data {} (pm, {}) at {}",
        index,
        LightPmC::SIZE,
        read.offset
    );
    let light: LightPmC = read.read_struct()?;
    trace!("{:#?}", light);

    assert_light(&light, read.prev)?;

    // read as a result of parent_count, but is always 0 (= world node index)
    let light_parent = read.read_u32()?;
    assert_that!("light parent", light_parent == 0, read.prev)?;

    Ok(Light {
        name: LIGHT_NAME.to_owned(),
        unk004: light.unk004,
        unk156: light.unk156,
        unk160: light.unk160,
        range: light.range,
        parent_ptr: light.parent_ptr,
        data_ptr,
    })
}

pub fn write(write: &mut CountingWriter<impl Write>, light: &Light, index: usize) -> Result<()> {
    debug!(
        "Writing light node data {} (pm, {}) at {}",
        index,
        LightPmC::SIZE,
        write.offset
    );
    let unk_combined = light.unk156 + light.unk160;

    let light = LightPmC {
        unk000: -0.5235988,
        unk004: light.unk004,
        zero008: Zeros::new(),
        unk136: 1.0,
        zero140: 0,
        zero144: 0,
        zero148: 0,
        zero152: 0,
        unk156: light.unk156,
        unk160: light.unk160,
        unk164: 1.0,
        unk168: 1.0,
        unk172: 1.0,
        unk176: 1.0,
        unk180: 1.0,
        unk184: 1.0,
        unk188: light.unk156,
        unk192: light.unk156,
        unk196: light.unk156,
        unk200: light.unk160,
        unk204: light.unk160,
        unk208: light.unk160,
        unk212: unk_combined,
        unk216: unk_combined,
        unk220: unk_combined,
        unk224: 2442,
        range: light.range,
        unk236: 1024.0,
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
