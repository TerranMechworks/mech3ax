use super::wrappers::WrapperPm;
use crate::flags::NodeBitFlags;
use crate::types::{NodeVariantPm, NodeVariantsPm};
use log::trace;
use mech3ax_api_types::{static_assert_size, BoundingBox, LodPm, Range, ReprSize as _};
use mech3ax_common::assert::assert_all_zero;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, bool_c, Result};
use std::io::{Read, Write};

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct Hide<T>(pub T);

impl<T> std::fmt::Debug for Hide<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("...")
    }
}

#[derive(Debug)]
#[repr(C)]
struct LodPmC {
    level: u32,             // 00
    range_near_sq: f32,     // 04
    range_far: f32,         // 08
    range_far_sq: f32,      // 12
    zero16: Hide<[u8; 44]>, // 16
    unk60: f32,             // 60
    unk64: f32,             // 64
    unk68: f32,             // 68
    unk72: f32,             // 72
    unk76: f32,             // 76
    one80: u32,             // 80
    unk84: u32,             // 84
    unk88: u32,             // 88
}
/*
level: u32,         // 00
1
range_far_sq: f32, // 04
4900.0
range_near: f32,     // 08
550.0
range_near_sq: f32,  // 12
302500.0 (550.0 ^ 2)

zero16: [u8; 44],   // 16

unk60: f32,         // 60
0.0
unk64: f32,         // 64
350.0
unk68: f32,         // 68
122500.0 (350.0 ^ 2)
unk72: f32,         // 72
549.0
unk76: f32,         // 76
301400.0 (549.0 ^ 2)

one80: u32,         // 80
unk84: u32,         // 84
unk88: u32,         // 88

*/
static_assert_size!(LodPmC, 92);

/*
ACTIVE = (1 << 2) BASE, DEFAULT
ALTITUDE_SURFACE = (1 << 3) DEFAULT
INTERSECT_SURFACE = (1 << 4) DEFAULT
UNK08 = (1 << 8)
UNK10 = (1 << 10)
TREE_VALID = (1 << 19) BASE, DEFAULT
ID_ZONE_CHECK = (1 << 24) BASE, DEFAULT
UNK25 = (1 << 25)
*/

const ALWAYS_PRESENT: NodeBitFlags = NodeBitFlags::from_bits_truncate(
    NodeBitFlags::BASE.bits() // MW
    | NodeBitFlags::UNK08.bits() // MW
    | NodeBitFlags::UNK10.bits() // MW
    | NodeBitFlags::ALTITUDE_SURFACE.bits() // PM (mechlib)
    | NodeBitFlags::INTERSECT_SURFACE.bits() // PM (mechlib)
    | NodeBitFlags::UNK25.bits(), // PM (mechlib)
);
#[allow(unused)]
const NEVER_PRESENT: NodeBitFlags = NodeBitFlags::from_bits_truncate(
    NodeBitFlags::LANDMARK.bits() // MW
        | NodeBitFlags::HAS_MESH.bits() // MW
        | NodeBitFlags::CAN_MODIFY.bits() // MW
        | NodeBitFlags::CLIP_TO.bits() // MW
        | NodeBitFlags::UNK28.bits() // MW
        | NodeBitFlags::INTERSECT_BBOX.bits() // PM (mechlib)
        | NodeBitFlags::TERRAIN.bits(), // PM (mechlib)
);

#[allow(clippy::collapsible_else_if)]
pub fn assert_variants(
    node: NodeVariantsPm,
    offset: u32,
    mesh_index_is_ptr: bool,
) -> Result<NodeVariantPm> {
    // cannot assert name
    // let const_flags = node.flags & (ALWAYS_PRESENT | NEVER_PRESENT);
    // assert_that!("lod flags", const_flags == ALWAYS_PRESENT, offset + 36)?;
    // variable
    assert_that!("lod flags", node.flags == ALWAYS_PRESENT, offset + 36)?;

    // mechlib only?
    assert_that!("lod field 044", node.unk044 == 1, offset + 44)?;
    /*
    if node.zone_id != ZONE_DEFAULT {
        assert_that!("lod zone id", 1 <= node.zone_id <= 80, offset + 48)?;
    }
    */
    // // mechlib only?
    // assert_that!("lod zone id", node.zone_id == ZONE_DEFAULT, offset + 48)?;

    assert_that!("lod data ptr", node.data_ptr != 0, offset + 56)?;
    if mesh_index_is_ptr {
        assert_that!("lod mesh index", node.mesh_index == 0, offset + 60)?;
    } else {
        assert_that!("lod mesh index", node.mesh_index == -1, offset + 60)?;
    }
    // // mechlib only?
    // assert_that!("lod area partition", node.area_partition == None, offset + 76)?;
    // must have one parent
    assert_that!("lod has parent", node.has_parent == true, offset + 84)?;
    // parent array ptr is already asserted
    // always has at least one child
    assert_that!("lod children count", 1 <= node.children_count <= 32, offset + 92)?;
    // children array ptr is already asserted
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
    Ok(NodeVariantPm::Lod(node))
}

fn assert_lod(lod: LodPmC, offset: u32) -> Result<(bool, Range, f32, f32)> {
    trace!("{:#?}", lod);
    let level = assert_that!("level", bool lod.level, offset + 0)?;

    assert_that!("range near sq", 0.0 <= lod.range_near_sq <= 1000.0 * 1000.0, offset + 4)?;
    let range_near = lod.range_near_sq.sqrt();
    assert_that!("range far", lod.range_far > 0.0, offset + 8)?;
    let expected = lod.range_far * lod.range_far;
    assert_that!("range far sq", lod.range_far_sq == expected, offset + 12)?;

    assert_all_zero("field 16", offset + 16, &lod.zero16.0)?;
    assert_that!("field 60", lod.unk60 == 0.0, offset + 60)?;

    assert_that!("field 64", lod.unk64 >= 0.0, offset + 64)?;
    let expected = lod.unk64 * lod.unk64;
    assert_that!("field 68", lod.unk68 == expected, offset + 68)?;

    assert_that!("field 72", lod.unk72 >= 0.0, offset + 72)?;
    let expected = lod.unk72 * lod.unk72;
    assert_that!("field 76", lod.unk76 == expected, offset + 76)?;

    assert_that!("field 80", lod.one80 == 1, offset + 80)?;

    // TODO: PM more data needed
    assert_that!("field 84", lod.unk84 == 0, offset + 84)?;
    assert_that!("field 89", lod.unk88 == 0, offset + 88)?;
    /*
    let unk84 = assert_that!("field 84", bool lod.unk84, offset + 84)?;
    let unk88 = if unk84 {
        assert_that!("field 88", lod.unk88 != 0, offset + 88)?;
        Some(lod.unk88)
    } else {
        assert_that!("field 88", lod.unk88 == 0, offset + 88)?;
        None
    };
    */

    Ok((
        level,
        Range {
            min: range_near,
            max: lod.range_far,
        },
        lod.unk64,
        lod.unk72,
    ))
}

pub fn read(
    read: &mut CountingReader<impl Read>,
    node: NodeVariantsPm,
) -> Result<WrapperPm<LodPm>> {
    let lod: LodPmC = read.read_struct()?;
    let (level, range, unk64, unk72) = assert_lod(lod, read.prev)?;

    let wrapped = LodPm {
        name: node.name,
        level,
        range,
        unk64,
        unk72,
        // flags: node.flags.into(),
        // zone_id: node.zone_id,
        // area_partition: node.area_partition,
        parent: 0,
        children: Vec::new(),
        data_ptr: node.data_ptr,
        parent_array_ptr: node.parent_array_ptr,
        children_array_ptr: node.children_array_ptr,
        unk164: node.unk164,
    };
    Ok(WrapperPm {
        wrapped,
        has_parent: false,
        children_count: node.children_count,
    })
}

pub fn make_variants(lod: &LodPm, mesh_index_is_ptr: bool) -> NodeVariantsPm {
    NodeVariantsPm {
        name: lod.name.clone(),
        // flags: NodeBitFlags::from(&lod.flags),
        flags: ALWAYS_PRESENT,
        unk044: 1,
        // zone_id: lod.zone_id,
        // zone_id: ZONE_DEFAULT,
        data_ptr: lod.data_ptr,
        mesh_index: if mesh_index_is_ptr { 0 } else { -1 },
        // area_partition: lod.area_partition,
        // area_partition: None,
        has_parent: true,
        parent_array_ptr: lod.parent_array_ptr,
        children_count: lod.children.len() as u16,
        children_array_ptr: lod.children_array_ptr,
        unk112: 2,
        unk116: BoundingBox::EMPTY,
        unk140: BoundingBox::EMPTY,
        unk164: lod.unk164,
    }
}

pub fn write(write: &mut CountingWriter<impl Write>, lod: &LodPm) -> Result<()> {
    write.write_struct(&LodPmC {
        level: bool_c!(lod.level),
        range_near_sq: lod.range.min * lod.range.min,
        range_far: lod.range.max,
        range_far_sq: lod.range.max * lod.range.max,
        zero16: Hide([0; 44]),
        unk60: 0.0,
        unk64: lod.unk64,
        unk68: lod.unk64 * lod.unk64,
        unk72: lod.unk72,
        unk76: lod.unk72 * lod.unk72,
        one80: 1,
        unk84: 0,
        unk88: 0,
    })?;
    Ok(())
}

pub fn size(lod: &LodPm) -> u32 {
    LodPmC::SIZE + 4 + 4 * lod.children.len() as u32
}
