use crate::pm::node::NodeVariantLodPm;
use crate::pm::wrappers::WrapperPm;
use bytemuck::{AnyBitPattern, NoUninit};
use log::{debug, trace};
use mech3ax_api_types::nodes::pm::Lod;
use mech3ax_api_types::{impl_as_bytes, AsBytes as _, Range};
use mech3ax_common::assert::assert_all_zero;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, bool_c, Result};
use mech3ax_debug::Zeros;
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
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
impl_as_bytes!(LodPmC, 92);

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
