use crate::mw::node::NodeVariantLodMw;
use crate::mw::wrappers::WrapperMw;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::Range;
use mech3ax_api_types::nodes::mw::Lod;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{Result, assert_that};
use mech3ax_types::{AsBytes as _, Bool32, Zeros, impl_as_bytes};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct LodMwC {
    level: Bool32,      // 00
    range_near_sq: f32, // 04
    range_far: f32,     // 08
    range_far_sq: f32,  // 12
    zero16: Zeros<44>,  // 16
    unk60: f32,         // 60
    unk64: f32,         // 64
    one68: u32,         // 68
    unk72: Bool32,      // 72
    unk76: u32,         // 76
}
impl_as_bytes!(LodMwC, 80);

fn assert_lod(lod: LodMwC, offset: usize) -> Result<(bool, Range, f32, Option<u32>)> {
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

pub(crate) fn read(
    read: &mut CountingReader<impl Read>,
    node: NodeVariantLodMw,
) -> Result<WrapperMw<Lod>> {
    let lod: LodMwC = read.read_struct()?;

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

pub(crate) fn write(write: &mut CountingWriter<impl Write>, lod: &Lod) -> Result<()> {
    let lod = LodMwC {
        level: lod.level.into(),
        range_near_sq: lod.range.min * lod.range.min,
        range_far: lod.range.max,
        range_far_sq: lod.range.max * lod.range.max,
        zero16: Zeros::new(),
        unk60: lod.unk60,
        unk64: lod.unk60 * lod.unk60,
        one68: 1,
        unk72: lod.unk76.is_some().into(),
        unk76: lod.unk76.unwrap_or(0),
    };
    write.write_struct(&lod)?;
    Ok(())
}

pub(crate) fn size(lod: &Lod) -> u32 {
    // Cast safety: truncation simply leads to incorrect size (TODO?)
    let children_length = lod.children.len() as u32;
    LodMwC::SIZE + 4 + 4 * children_length
}
