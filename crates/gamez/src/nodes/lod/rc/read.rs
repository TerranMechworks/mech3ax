use super::LodRcC;
use mech3ax_api_types::gamez::nodes::Lod;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{chk, Result};
use std::io::Read;

pub(crate) fn read(read: &mut CountingReader<impl Read>) -> Result<Lod> {
    let lod: LodRcC = read.read_struct()?;

    let lod = assert_lod(&lod, read.prev)?;

    // let parent = if node.has_parent {
    //     Some(read.read_u32()?)
    // } else {
    //     None
    // };
    // let children = read_child_indices(read, node.children_count)?;

    Ok(lod)
}

fn assert_lod(lod: &LodRcC, offset: usize) -> Result<Lod> {
    // TODO
    // chk!(offset, lod.field00 == 1)?;
    chk!(offset, lod.range_near_sq >= 0.0)?;
    // TODO
    // chk!(offset, lod.range_far == 0.0)?;
    let range_far_sq = lod.range_far * lod.range_far;
    chk!(offset, lod.range_far_sq == range_far_sq)?;
    chk!(offset, lod.field16 == 0.0)?;
    chk!(offset, lod.field20 == 0.0)?;
    chk!(offset, lod.field24 == 0.0)?;
    chk!(offset, lod.field28 == 0.0)?;
    chk!(offset, lod.field32 == 0.0)?;
    chk!(offset, lod.field36 == 0.0)?;
    chk!(offset, lod.field40 == 0.0)?;
    chk!(offset, lod.field44 == 0.0)?;
    chk!(offset, lod.field48 == 0)?;
    chk!(offset, lod.field52 == 0.0)?;
    chk!(offset, lod.field56 == 0.0)?;
    // TODO
    chk!(offset, lod.field60 == 0.0)?;
    // TODO (field60 * field60)
    chk!(offset, lod.field64 == 0.0)?;
    chk!(offset, lod.field68 == 1)?;
    // TODO
    // chk!(offset, lod.field72 == 0.0)?;
    // chk!(offset, lod.field76 == 0.0)?;

    Ok(Lod {})
}
