use super::node::{NodeVariantLodRc, NodeVariantRc, NodeVariantsRc};
use crate::flags::NodeBitFlags;
use crate::types::ZONE_DEFAULT;
use log::{debug, trace};
use mech3ax_api_types::nodes::rc::Lod;
use mech3ax_api_types::nodes::BoundingBox;
use mech3ax_api_types::{static_assert_size, Range, ReprSize as _};
use mech3ax_common::assert::assert_all_zero;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, bool_c, Result};
use mech3ax_debug::Zeros;
use std::io::{Read, Write};

#[derive(Debug)]
#[repr(C)]
struct LodRcC {
    level: u32,         // 00
    range_near_sq: f32, // 04
    range_far: f32,     // 08
    range_far_sq: f32,  // 12
    zero16: Zeros<44>,  // 16
    unk60: f32,         // 60
    unk64: f32,         // 64
    one68: u32,         // 68
    unk72: u32,         // 72
    unk76: u32,         // 76
}
static_assert_size!(LodRcC, 80);

const ALWAYS_PRESENT: NodeBitFlags = NodeBitFlags::from_bits_truncate(
    0
    | NodeBitFlags::ACTIVE.bits()
    | NodeBitFlags::ALTITUDE_SURFACE.bits()
    | NodeBitFlags::INTERSECT_SURFACE.bits()
    // | NodeBitFlags::INTERSECT_BBOX.bits()
    // | NodeBitFlags::LANDMARK.bits()
    // | NodeBitFlags::UNK08.bits()
    // | NodeBitFlags::HAS_MESH.bits()
    // | NodeBitFlags::UNK10.bits()
    // | NodeBitFlags::TERRAIN.bits()
    // | NodeBitFlags::CAN_MODIFY.bits()
    // | NodeBitFlags::CLIP_TO.bits()
    | NodeBitFlags::TREE_VALID.bits()
    | NodeBitFlags::ID_ZONE_CHECK.bits()
    // | NodeBitFlags::UNK25.bits()
    // | NodeBitFlags::UNK28.bits()
    | 0,
);
const VARIABLE_FLAGS: NodeBitFlags = NodeBitFlags::from_bits_truncate(
    0
    // | NodeBitFlags::ACTIVE.bits()
    // | NodeBitFlags::ALTITUDE_SURFACE.bits()
    // | NodeBitFlags::INTERSECT_SURFACE.bits()
    // | NodeBitFlags::INTERSECT_BBOX.bits()
    // | NodeBitFlags::LANDMARK.bits()
    | NodeBitFlags::UNK08.bits()
    // | NodeBitFlags::HAS_MESH.bits()
    | NodeBitFlags::UNK10.bits()
    // | NodeBitFlags::TERRAIN.bits()
    // | NodeBitFlags::CAN_MODIFY.bits()
    // | NodeBitFlags::CLIP_TO.bits()
    // | NodeBitFlags::TREE_VALID.bits()
    // | NodeBitFlags::ID_ZONE_CHECK.bits()
    // | NodeBitFlags::UNK25.bits()
    // | NodeBitFlags::UNK28.bits()
    | 0,
);

const BORKED_UNK044: &[u32; 6] = &[
    0x0189AF70, 0x018B16A0, 0x018C2C30, 0x018C54C0, 0x018CC320, 0x018F4930,
];

pub fn assert_variants(node: NodeVariantsRc, offset: u32) -> Result<NodeVariantRc> {
    debug!("LoD: {:#?}", node);
    // cannot assert name
    let const_flags = node.flags & !VARIABLE_FLAGS;
    assert_that!("lod flags", const_flags == ALWAYS_PRESENT, offset + 36)?;
    if node.flags.contains(NodeBitFlags::UNK08) {
        let has_10 = node.flags.contains(NodeBitFlags::UNK10);
        assert_that!("lod flags 10", has_10 == true, offset + 36)?;
    }
    // zero040 (40) already asserted
    // there's six cases where unk044 is 0, so we'll use a dirty hack
    if BORKED_UNK044.contains(&node.data_ptr) {
        assert_that!("lod field 044", node.unk044 == 0, offset + 44)?;
    } else {
        assert_that!("lod field 044", node.unk044 == 4, offset + 44)?;
    }
    if node.zone_id != ZONE_DEFAULT {
        assert_that!("lod zone id", 0 <= node.zone_id <= 80, offset + 48)?;
    }
    // node_type (52) already asserted
    assert_that!("lod data ptr", node.data_ptr != 0, offset + 56)?;
    assert_that!("lod mesh index", node.mesh_index == -1, offset + 60)?;
    // environment_data (64) already asserted
    // action_priority (68) already asserted
    // action_callback (72) already asserted
    assert_that!(
        "display area partition",
        node.area_partition == None,
        offset + 76
    )?;
    // has_parent (84) is variable
    // parent_array_ptr (88) already asserted
    // children_count (92) is variable
    // children_array_ptr (96) already asserted
    // zero100 (100) already asserted
    // zero104 (104) already asserted
    // zero108 (108) already asserted
    // zero112 (112) already asserted
    assert_that!(
        "lod bbox 2",
        node.unk140 == BoundingBox::EMPTY,
        offset + 140
    )?;
    assert_that!("lod bbox 3", node.unk164 == node.unk116, offset + 164)?;
    // zero188 (188) already asserted
    Ok(NodeVariantRc::Lod(NodeVariantLodRc {
        name: node.name,
        flags: node.flags,
        zone_id: node.zone_id,
        data_ptr: node.data_ptr,
        has_parent: node.has_parent,
        parent_array_ptr: node.parent_array_ptr,
        children_count: node.children_count,
        children_array_ptr: node.children_array_ptr,
        unk116: node.unk116,
    }))
}

pub fn make_variants(lod: &Lod) -> Result<NodeVariantsRc> {
    let children_count = assert_len!(u32, lod.children.len(), "lod children")?;
    let unk044 = if BORKED_UNK044.contains(&lod.data_ptr) {
        0
    } else {
        4
    };
    Ok(NodeVariantsRc {
        name: lod.name.clone(),
        flags: NodeBitFlags::from(&lod.flags),
        unk044,
        zone_id: lod.zone_id,
        data_ptr: lod.data_ptr,
        mesh_index: -1,
        area_partition: None,
        has_parent: lod.parent.is_some(),
        parent_array_ptr: lod.parent_array_ptr,
        children_count,
        children_array_ptr: lod.children_array_ptr,
        unk116: lod.unk116,
        unk140: BoundingBox::EMPTY,
        unk164: lod.unk116,
    })
}

fn assert_lod(lod: &LodRcC, offset: u32) -> Result<(bool, Range, f32, Option<u32>)> {
    let level = assert_that!("lod level", bool lod.level, offset + 0)?;
    assert_that!("lod range near sq", 0.0 <= lod.range_near_sq <= 1000.0 * 1000.0, offset + 4)?;
    let range_near = lod.range_near_sq.sqrt();
    assert_that!("lod range far", lod.range_far > 0.0, offset + 8)?;
    let expected = lod.range_far * lod.range_far;
    assert_that!(
        "lod range far sq",
        lod.range_far_sq == expected,
        offset + 12
    )?;

    assert_all_zero("lod field 16", offset + 16, &lod.zero16.0)?;

    assert_that!("lod field 60", lod.unk60 >= 0.0, offset + 60)?;
    let expected = lod.unk60 * lod.unk60;
    assert_that!("lod field 64", lod.unk64 == expected, offset + 64)?;
    assert_that!("lod field 68", lod.one68 == 1, offset + 68)?;
    let unk72 = assert_that!("lod field 72", bool lod.unk72, offset + 72)?;
    let unk76 = if unk72 {
        assert_that!("lod field 76", lod.unk76 != 0, offset + 76)?;
        Some(lod.unk76)
    } else {
        assert_that!("lod field 76", lod.unk76 == 0, offset + 76)?;
        None
    };

    Ok((
        level,
        Range {
            min: range_near,
            max: lod.range_far,
        },
        lod.unk60,
        unk76,
    ))
}

pub fn read(
    read: &mut CountingReader<impl Read>,
    node: NodeVariantLodRc,
    index: usize,
) -> Result<Lod> {
    debug!(
        "Reading lod node data {} (rc, {}) at {}",
        index,
        LodRcC::SIZE,
        read.offset
    );
    let lod: LodRcC = read.read_struct()?;
    trace!("{:#?}", lod);

    let (level, range, unk60, unk76) = assert_lod(&lod, read.prev)?;

    let parent = if node.has_parent {
        Some(read.read_u32()?)
    } else {
        None
    };

    debug!(
        "Reading lod {} x children {} (rc) at {}",
        node.children_count, index, read.offset
    );
    let children = (0..node.children_count)
        .map(|_| read.read_u32())
        .collect::<std::io::Result<Vec<_>>>()?;

    Ok(Lod {
        name: node.name,
        level,
        range,
        unk60,
        unk76,
        flags: node.flags.into(),
        zone_id: node.zone_id,
        parent,
        children,
        data_ptr: node.data_ptr,
        parent_array_ptr: node.parent_array_ptr,
        children_array_ptr: node.children_array_ptr,
        unk116: node.unk116,
    })
}

pub fn write(write: &mut CountingWriter<impl Write>, lod: &Lod, index: usize) -> Result<()> {
    debug!(
        "Writing lod node data {} (rc, {}) at {}",
        index,
        LodRcC::SIZE,
        write.offset
    );
    let lodc = LodRcC {
        level: bool_c!(lod.level),
        range_near_sq: lod.range.min * lod.range.min,
        range_far: lod.range.max,
        range_far_sq: lod.range.max * lod.range.max,
        zero16: Zeros::new(),
        unk60: lod.unk60,
        unk64: lod.unk60 * lod.unk60,
        one68: 1,
        unk72: bool_c!(lod.unk76.is_some()),
        unk76: lod.unk76.unwrap_or(0),
    };
    trace!("{:#?}", lodc);
    write.write_struct(&lodc)?;

    if let Some(parent) = lod.parent {
        write.write_u32(parent)?;
    }

    debug!(
        "Writing lod {} x children {} (rc) at {}",
        lod.children.len(),
        index,
        write.offset
    );
    for child in &lod.children {
        write.write_u32(*child)?;
    }

    Ok(())
}

pub fn size(lod: &Lod) -> u32 {
    let parent_size = if lod.parent.is_some() { 4 } else { 0 };
    // Cast safety: truncation simply leads to incorrect size (TODO?)
    let children_length = lod.children.len() as u32;
    LodRcC::SIZE + parent_size + 4 * children_length
}