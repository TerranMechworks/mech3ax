use super::node::{NodeVariantLodPm, NodeVariantPm, NodeVariantsPm};
use super::wrappers::WrapperPm;
use crate::flags::NodeBitFlags;
use log::{debug, trace};
use mech3ax_api_types::nodes::pm::Lod;
use mech3ax_api_types::nodes::BoundingBox;
use mech3ax_api_types::{static_assert_size, Range, ReprSize as _};
use mech3ax_common::assert::assert_all_zero;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, bool_c, Result};
use mech3ax_debug::Zeros;
use std::io::{Read, Write};

#[derive(Debug)]
#[repr(C)]
struct LodPmC {
    level: u32,         // 00
    range_near_sq: f32, // 04
    range_far: f32,     // 08
    range_far_sq: f32,  // 12
    zero16: Zeros<48>,  // 16
    unk64: f32,         // 64
    unk68: f32,         // 68
    unk72: f32,         // 72
    unk76: f32,         // 76
    one80: u32,         // 80
    unk84: u32,         // 84
    unk88: u32,         // 88
}
static_assert_size!(LodPmC, 92);

const ALWAYS_PRESENT: NodeBitFlags = NodeBitFlags::from_bits_truncate(
    0
    | NodeBitFlags::ACTIVE.bits()
    | NodeBitFlags::ALTITUDE_SURFACE.bits()
    | NodeBitFlags::INTERSECT_SURFACE.bits()
    // | NodeBitFlags::INTERSECT_BBOX.bits()
    // | NodeBitFlags::LANDMARK.bits()
    | NodeBitFlags::UNK08.bits()
    // | NodeBitFlags::HAS_MESH.bits()
    | NodeBitFlags::UNK10.bits()
    // | NodeBitFlags::TERRAIN.bits()
    // | NodeBitFlags::CAN_MODIFY.bits()
    // | NodeBitFlags::CLIP_TO.bits()
    | NodeBitFlags::TREE_VALID.bits()
    | NodeBitFlags::ID_ZONE_CHECK.bits()
    | NodeBitFlags::UNK25.bits()
    // | NodeBitFlags::UNK28.bits()
    | 0,
);

pub fn assert_variants(
    node: NodeVariantsPm,
    offset: u32,
    mesh_index_is_ptr: bool,
) -> Result<NodeVariantPm> {
    // cannot assert name
    assert_that!("lod flags", node.flags == ALWAYS_PRESENT, offset + 36)?;
    // zero040 (40) already asserted
    assert_that!("lod field 044", node.unk044 == 1, offset + 44)?;
    // zone_id (48) is variable
    // node_type (52) already asserted
    assert_that!("lod data ptr", node.data_ptr != 0, offset + 56)?;
    if mesh_index_is_ptr {
        assert_that!("lod mesh index", node.mesh_index == 0, offset + 60)?;
    } else {
        assert_that!("lod mesh index", node.mesh_index == -1, offset + 60)?;
    }
    // environment_data (64) already asserted
    // action_priority (68) already asserted
    // action_callback (72) already asserted
    assert_that!(
        "lod area partition",
        node.area_partition == None,
        offset + 76
    )?;
    // must have one parent
    assert_that!("lod has parent", node.has_parent == true, offset + 84)?;
    // parent_array_ptr (88) already asserted
    // always has at least one child
    assert_that!("lod children count", 1 <= node.children_count <= 32, offset + 86)?;
    // children_array_ptr (92) already asserted
    // zero096 (96) already asserted
    // zero100 (100) already asserted
    // zero104 (104) already asserted
    // zero108 (108) already asserted
    assert_that!("lod field 112", node.unk112 == 2, offset + 112)?;
    assert_that!(
        "lod bbox 1",
        node.unk116 == BoundingBox::EMPTY,
        offset + 116
    )?;
    assert_that!(
        "lod bbox 2",
        node.unk140 == BoundingBox::EMPTY,
        offset + 140
    )?;
    assert_that!(
        "lod bbox 3",
        node.unk164 != BoundingBox::EMPTY,
        offset + 164
    )?;
    // zero188 (188) already asserted
    // zero192 (192) already asserted
    assert_that!("lod field 196", node.unk196 == 160, offset + 196)?;
    // zero200 (200) already asserted
    // zero204 (204) already asserted
    Ok(NodeVariantPm::Lod(NodeVariantLodPm {
        name: node.name,
        flags: node.flags,
        zone_id: node.zone_id,
        data_ptr: node.data_ptr,
        parent_array_ptr: node.parent_array_ptr,
        children_count: node.children_count,
        children_array_ptr: node.children_array_ptr,
        unk164: node.unk164,
    }))
}

pub fn make_variants(lod: &Lod, mesh_index_is_ptr: bool) -> Result<NodeVariantsPm> {
    let children_count = assert_len!(u16, lod.children.len(), "lod children")?;
    Ok(NodeVariantsPm {
        name: lod.name.clone(),
        flags: ALWAYS_PRESENT,
        unk044: 1,
        zone_id: lod.zone_id,
        data_ptr: lod.data_ptr,
        mesh_index: if mesh_index_is_ptr { 0 } else { -1 },
        area_partition: None,
        has_parent: true,
        parent_array_ptr: lod.parent_array_ptr,
        children_count,
        children_array_ptr: lod.children_array_ptr,
        unk112: 2,
        unk116: BoundingBox::EMPTY,
        unk140: BoundingBox::EMPTY,
        unk164: lod.unk164,
        unk196: 160,
    })
}

fn assert_lod(lod: &LodPmC, offset: u32) -> Result<(bool, Range)> {
    trace!("{:#?}", lod);
    let level = assert_that!("level", bool lod.level, offset + 0)?;

    assert_that!("range near sq", 0.0 <= lod.range_near_sq <= 1000.0 * 1000.0, offset + 4)?;
    let range_near = lod.range_near_sq.sqrt();
    assert_that!("range far", lod.range_far > 0.0, offset + 8)?;
    let expected = lod.range_far * lod.range_far;
    assert_that!("range far sq", lod.range_far_sq == expected, offset + 12)?;

    assert_all_zero("field 16", offset + 16, &lod.zero16.0)?;

    assert_that!("field 64", lod.unk64 >= 0.0, offset + 64)?;
    let expected = lod.unk64 * lod.unk64;
    assert_that!("field 68", lod.unk68 == expected, offset + 68)?;

    assert_that!("field 72", lod.unk72 >= 0.0, offset + 72)?;
    let expected = lod.unk72 * lod.unk72;
    assert_that!("field 76", lod.unk76 == expected, offset + 76)?;

    assert_that!("field 80", lod.one80 == 1, offset + 80)?;

    assert_that!("field 84", lod.unk84 == 0, offset + 84)?;
    assert_that!("field 89", lod.unk88 == 0, offset + 88)?;

    let range = Range {
        min: range_near,
        max: lod.range_far,
    };
    Ok((level, range))
}

pub fn read(
    read: &mut CountingReader<impl Read>,
    node: NodeVariantLodPm,
    index: usize,
) -> Result<WrapperPm<Lod>> {
    debug!(
        "Reading lod node data {} (pm, {}) at {}",
        index,
        LodPmC::SIZE,
        read.offset
    );
    let lod: LodPmC = read.read_struct()?;
    trace!("{:#?}", lod);

    let (level, range) = assert_lod(&lod, read.prev)?;

    let wrapped = Lod {
        name: node.name,
        level,
        range,
        unk64: lod.unk64,
        unk72: lod.unk72,
        zone_id: node.zone_id,
        parent: 0,            // to be filled in later
        children: Vec::new(), // to be filled in later
        unk164: node.unk164,
        parent_array_ptr: node.parent_array_ptr,
        children_array_ptr: node.children_array_ptr,
        data_ptr: node.data_ptr,
        node_index: 0, // to be filled in for gamez
    };
    Ok(WrapperPm {
        wrapped,
        has_parent: false,
        children_count: node.children_count,
    })
}

pub fn write(write: &mut CountingWriter<impl Write>, lod: &Lod, index: usize) -> Result<()> {
    debug!(
        "Writing lod node data {} (pm, {}) at {}",
        index,
        LodPmC::SIZE,
        write.offset
    );
    let lod = LodPmC {
        level: bool_c!(lod.level),
        range_near_sq: lod.range.min * lod.range.min,
        range_far: lod.range.max,
        range_far_sq: lod.range.max * lod.range.max,
        zero16: Zeros::new(),
        unk64: lod.unk64,
        unk68: lod.unk64 * lod.unk64,
        unk72: lod.unk72,
        unk76: lod.unk72 * lod.unk72,
        one80: 1,
        unk84: 0,
        unk88: 0,
    };
    trace!("{:#?}", lod);
    write.write_struct(&lod)?;
    Ok(())
}
