use super::node::{NodeVariantMw, NodeVariantsMw};
use crate::flags::NodeBitFlags;
use crate::types::ZONE_DEFAULT;
use log::{debug, trace};
use mech3ax_api_types::nodes::mw::Light;
use mech3ax_api_types::nodes::BoundingBox;
use mech3ax_api_types::{static_assert_size, Color, Range, ReprSize as _, Vec3};
use mech3ax_common::assert::assert_all_zero;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::light::LightFlags;
use mech3ax_common::{assert_that, assert_with_msg, Result};
use mech3ax_debug::Zeros;
use std::io::{Read, Write};

#[derive(Debug)]
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
    flags: u32,          // 176
    range: Range,        // 180
    range_near_sq: f32,  // 188
    range_far_sq: f32,   // 192
    range_inv: f32,      // 196
    parent_count: u32,   // 200
    parent_ptr: u32,     // 204
}
static_assert_size!(LightMwC, 208);

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

pub fn assert_variants(node: NodeVariantsMw, offset: u32) -> Result<NodeVariantMw> {
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
    // children_array_ptr (96) already asserted
    // zero100 (100) already asserted
    // zero104 (104) already asserted
    // zero108 (108) already asserted
    // zero112 (112) already asserted
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
    Ok(NodeVariantMw::Light {
        data_ptr: node.data_ptr,
    })
}

fn assert_light(light: &LightMwC, offset: u32) -> Result<()> {
    assert_that!(
        "translation",
        light.translation == Vec3::DEFAULT,
        offset + 12
    )?;
    assert_all_zero("field 024", offset + 24, &light.zero024.0)?;

    assert_that!("field 136", light.one136 == 1.0, offset + 136)?;
    assert_that!("field 140", light.zero140 == 0.0, offset + 140)?;
    assert_that!("field 144", light.zero144 == 0.0, offset + 144)?;
    assert_that!("field 148", light.zero148 == 0.0, offset + 148)?;
    assert_that!("field 152", light.zero152 == 0.0, offset + 152)?;

    assert_that!("diffuse", 0.0 <= light.diffuse <= 1.0, offset + 156)?;
    assert_that!("ambient", 0.0 <= light.ambient <= 1.0, offset + 160)?;

    assert_that!("color", light.color == Color::WHITE_NORM, offset + 164)?;

    let flags = LightFlags::from_bits(light.flags).ok_or_else(|| {
        assert_with_msg!(
            "Expected valid light flags, but was 0x{:08X} (at {})",
            light.flags,
            offset + 176
        )
    })?;
    assert_that!("flag", flags == LightFlags::DEFAULT, offset + 176)?;

    assert_that!("range near", light.range.min > 0.0, offset + 180)?;
    assert_that!("range far", light.range.max > light.range.min, offset + 184)?;
    let expected = light.range.min * light.range.min;
    assert_that!(
        "range near sq",
        light.range_near_sq == expected,
        offset + 188
    )?;
    let expected = light.range.max * light.range.max;
    assert_that!("range far sq", light.range_far_sq == expected, offset + 192)?;
    let expected = 1.0 / (light.range.max - light.range.min);
    assert_that!("range inv", light.range_inv == expected, offset + 196)?;

    assert_that!("parent count", light.parent_count == 1, offset + 200)?;
    assert_that!("parent ptr", light.parent_ptr != 0, offset + 204)?;
    Ok(())
}

pub fn read(read: &mut CountingReader<impl Read>, data_ptr: u32, index: usize) -> Result<Light> {
    debug!(
        "Reading light node data {} (mw, {}) at {}",
        index,
        LightMwC::SIZE,
        read.offset
    );
    let light: LightMwC = read.read_struct()?;
    trace!("{:#?}", light);

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

pub fn make_variants(light: &Light) -> NodeVariantsMw {
    NodeVariantsMw {
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
        unk116: BBOX_LIGHT,
        unk140: BoundingBox::EMPTY,
        unk164: BoundingBox::EMPTY,
        unk196: 0,
    }
}

pub fn write(write: &mut CountingWriter<impl Write>, light: &Light, index: usize) -> Result<()> {
    debug!(
        "Writing light node data {} (mw, {}) at {}",
        index,
        LightMwC::SIZE,
        write.offset
    );
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
        flags: LightFlags::DEFAULT.bits(),
        range: light.range,
        range_near_sq: light.range.min * light.range.min,
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
    LightMwC::SIZE + 4
}
