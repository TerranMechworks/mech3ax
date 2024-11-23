use crate::rc::node::NodeVariantLodRc;
use bytemuck::{AnyBitPattern, NoUninit};
use log::{debug, trace};
use mech3ax_api_types::nodes::rc::Lod;
use mech3ax_api_types::Range;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, bool_c, Result};
use mech3ax_types::Zeros;
use mech3ax_types::{impl_as_bytes, AsBytes as _};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
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
impl_as_bytes!(LodRcC, 80);

fn assert_lod(lod: &LodRcC, offset: usize) -> Result<(bool, Range, f32, Option<u32>)> {
    let level = assert_that!("lod level", bool lod.level, offset + 0)?;
    assert_that!("lod range near sq", 0.0 <= lod.range_near_sq <= 1000.0 * 1000.0, offset + 4)?;
    let range_near = lod.range_near_sq.sqrt();
    assert_that!("lod range far", lod.range_far >= 0.0, offset + 8)?;
    let expected = lod.range_far * lod.range_far;
    assert_that!(
        "lod range far sq",
        lod.range_far_sq == expected,
        offset + 12
    )?;

    assert_that!("lod field 16", zero lod.zero16, offset + 16)?;

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
    for child in lod.children.iter().copied() {
        write.write_u32(child)?;
    }

    Ok(())
}

pub fn size(lod: &Lod) -> u32 {
    let parent_size = if lod.parent.is_some() { 4 } else { 0 };
    // Cast safety: truncation simply leads to incorrect size (TODO?)
    let children_length = lod.children.len() as u32;
    LodRcC::SIZE + parent_size + 4 * children_length
}
