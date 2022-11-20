use super::wrappers::WrapperMw;
use crate::flags::NodeBitFlags;
use crate::types::{NodeVariantMw, NodeVariantsMw, ZONE_DEFAULT};
use log::{debug, trace};
use mech3ax_api_types::gamez::{BoundingBox, Lod};
use mech3ax_api_types::{static_assert_size, Range, ReprSize as _};
use mech3ax_common::assert::assert_all_zero;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, bool_c, Result};
use mech3ax_debug::Zeros;
use std::io::{Read, Write};

#[derive(Debug)]
#[repr(C)]
struct LodMwC {
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
static_assert_size!(LodMwC, 80);

const ALWAYS_PRESENT: NodeBitFlags = NodeBitFlags::from_bits_truncate(
    NodeBitFlags::BASE.bits() | NodeBitFlags::UNK08.bits() | NodeBitFlags::UNK10.bits(),
);
const NEVER_PRESENT: NodeBitFlags = NodeBitFlags::from_bits_truncate(
    NodeBitFlags::LANDMARK.bits()
        | NodeBitFlags::HAS_MESH.bits()
        | NodeBitFlags::CAN_MODIFY.bits()
        | NodeBitFlags::CLIP_TO.bits()
        | NodeBitFlags::UNK28.bits(),
);

pub fn assert_variants(node: NodeVariantsMw, offset: u32) -> Result<NodeVariantMw> {
    // cannot assert name
    let const_flags = node.flags & (ALWAYS_PRESENT | NEVER_PRESENT);
    assert_that!("lod flags", const_flags == ALWAYS_PRESENT, offset + 36)?;
    // variable
    /*
    const ALTITUDE_SURFACE = 1 << 3;
    const INTERSECT_SURFACE = 1 << 4;
    const INTERSECT_BBOX = 1 << 5;
    const TERRAIN = 1 << 15;
    const UNK25 = 1 << 25;
    */

    assert_that!("lod field 044", node.unk044 == 1, offset + 44)?;
    if node.zone_id != ZONE_DEFAULT {
        assert_that!("lod zone id", 1 <= node.zone_id <= 80, offset + 48)?;
    }
    assert_that!("lod data ptr", node.data_ptr != 0, offset + 56)?;
    assert_that!("lod mesh index", node.mesh_index == -1, offset + 60)?;
    // must have one parent
    assert_that!("lod has parent", node.has_parent == true, offset + 84)?;
    // parent array ptr is already asserted
    // always has at least one child
    assert_that!("lod children count", 1 <= node.children_count <= 32, offset + 92)?;
    // children array ptr is already asserted
    assert_that!(
        "lod bbox 1",
        node.unk116 != BoundingBox::EMPTY,
        offset + 116
    )?;
    assert_that!(
        "lod bbox 2",
        node.unk140 == BoundingBox::EMPTY,
        offset + 140
    )?;
    assert_that!("lod bbox 3", node.unk164 == node.unk116, offset + 164)?;
    assert_that!("lod field 196", node.unk196 == 160, offset + 196)?;
    Ok(NodeVariantMw::Lod(node))
}

fn assert_lod(lod: LodMwC, offset: u32) -> Result<(bool, Range, f32, Option<u32>)> {
    let level = assert_that!("level", bool lod.level, offset + 0)?;
    assert_that!("range near sq", 0.0 <= lod.range_near_sq <= 1000.0 * 1000.0, offset + 4)?;
    let range_near = lod.range_near_sq.sqrt();
    assert_that!("range far", lod.range_far > 0.0, offset + 8)?;
    let expected = lod.range_far * lod.range_far;
    assert_that!("range far sq", lod.range_far_sq == expected, offset + 12)?;

    assert_all_zero("field 16", offset + 16, &lod.zero16.0)?;

    assert_that!("field 60", lod.unk60 >= 0.0, offset + 60)?;
    let expected = lod.unk60 * lod.unk60;
    assert_that!("field 64", lod.unk64 == expected, offset + 64)?;
    assert_that!("field 68", lod.one68 == 1, offset + 68)?;
    let unk72 = assert_that!("field 72", bool lod.unk72, offset + 72)?;
    let unk76 = if unk72 {
        assert_that!("field 76", lod.unk76 != 0, offset + 76)?;
        Some(lod.unk76)
    } else {
        assert_that!("field 76", lod.unk76 == 0, offset + 76)?;
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
    node: NodeVariantsMw,
    index: usize,
) -> Result<WrapperMw<Lod>> {
    debug!(
        "Reading lod node data {} (mw, {}) at {}",
        index,
        LodMwC::SIZE,
        read.offset
    );
    let lod: LodMwC = read.read_struct()?;
    trace!("{:#?}", lod);

    let (level, range, unk60, unk76) = assert_lod(lod, read.prev)?;

    let wrapped = Lod {
        name: node.name,
        level,
        range,
        unk60,
        unk76,
        flags: node.flags.into(),
        zone_id: node.zone_id,
        area_partition: node.area_partition,
        parent: 0,
        children: Vec::new(),
        data_ptr: node.data_ptr,
        parent_array_ptr: node.parent_array_ptr,
        children_array_ptr: node.children_array_ptr,
        unk116: node.unk116,
    };
    Ok(WrapperMw {
        wrapped,
        has_parent: false,
        children_count: node.children_count,
    })
}

pub fn make_variants(lod: &Lod) -> Result<NodeVariantsMw> {
    let children_count = assert_len!(u32, lod.children.len(), "lod children")?;
    Ok(NodeVariantsMw {
        name: lod.name.clone(),
        flags: NodeBitFlags::from(&lod.flags),
        unk044: 1,
        zone_id: lod.zone_id,
        data_ptr: lod.data_ptr,
        mesh_index: -1,
        area_partition: lod.area_partition,
        has_parent: true,
        parent_array_ptr: lod.parent_array_ptr,
        children_count,
        children_array_ptr: lod.children_array_ptr,
        unk116: lod.unk116,
        unk140: BoundingBox::EMPTY,
        unk164: lod.unk116,
        unk196: 160,
    })
}

pub fn write(write: &mut CountingWriter<impl Write>, lod: &Lod, index: usize) -> Result<()> {
    debug!(
        "Writing lod node data {} (mw, {}) at {}",
        index,
        LodMwC::SIZE,
        write.offset
    );
    let lod = LodMwC {
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
    trace!("{:#?}", lod);
    write.write_struct(&lod)?;
    Ok(())
}

pub fn size(lod: &Lod) -> u32 {
    // Cast safety: truncation simply leads to incorrect size (TODO?)
    let children_length = lod.children.len() as u32;
    LodMwC::SIZE + 4 + 4 * children_length
}
