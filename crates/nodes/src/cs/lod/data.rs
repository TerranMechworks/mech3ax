use crate::cs::node::NodeVariantLodCs;
use bytemuck::{AnyBitPattern, NoUninit};
use log::{debug, trace};
use mech3ax_api_types::nodes::cs::Lod;
use mech3ax_api_types::{static_assert_size, Range, ReprSize as _};
use mech3ax_common::assert::assert_all_zero;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, bool_c, Result};
use mech3ax_debug::Zeros;
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct LodCsC {
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
static_assert_size!(LodCsC, 92);

fn assert_lod(lod: &LodCsC, offset: u32) -> Result<(bool, Range)> {
    trace!("{:#?}", lod);
    let level = assert_that!("level", bool lod.level, offset + 0)?;

    assert_that!("range near sq", 0.0 <= lod.range_near_sq <= 3000.0 * 3000.0, offset + 4)?;
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
    node: NodeVariantLodCs,
    node_index: u32,
    index: usize,
) -> Result<Lod> {
    debug!(
        "Reading lod node data {} (cs, {}) at {}",
        index,
        LodCsC::SIZE,
        read.offset
    );
    let lod: LodCsC = read.read_struct()?;
    trace!("{:#?}", lod);

    let (level, range) = assert_lod(&lod, read.prev)?;

    let parent = read.read_u32()?;
    debug!(
        "Reading node {} children x{} (cs) at {}",
        index, node.children_count, read.offset
    );
    let children = (0..node.children_count)
        .map(|_| read.read_u32())
        .collect::<std::io::Result<Vec<_>>>()?;

    Ok(Lod {
        name: node.name,
        level,
        range,
        unk64: lod.unk64,
        unk72: lod.unk72,
        parent,
        children,
        flags_unk03: node.flags_unk03,
        flags_unk04: node.flags_unk04,
        flags_unk07: node.flags_unk07,
        unk040: node.unk040,
        zone_id: node.zone_id,
        data_ptr: node.data_ptr,
        parent_array_ptr: node.parent_array_ptr,
        children_array_ptr: node.children_array_ptr,
        unk164: node.unk164,
        node_index,
    })
}

pub fn write(write: &mut CountingWriter<impl Write>, lod: &Lod, index: usize) -> Result<()> {
    debug!(
        "Writing lod node data {} (cs, {}) at {}",
        index,
        LodCsC::SIZE,
        write.offset
    );
    let lodc = LodCsC {
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
    trace!("{:#?}", lodc);
    write.write_struct(&lodc)?;

    write.write_u32(lod.parent)?;
    debug!(
        "Writing node {} children x{} (cs) at {}",
        index,
        lod.children.len(),
        write.offset
    );
    for child in lod.children.iter().copied() {
        write.write_u32(child)?;
    }

    Ok(())
}
