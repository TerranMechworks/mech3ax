use super::LodRcC;
use mech3ax_api_types::Range;
use mech3ax_api_types::gamez::nodes::Lod;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{Result, chk};
use std::io::Read;

pub(crate) fn read(read: &mut CountingReader<impl Read>) -> Result<Lod> {
    let lod: LodRcC = read.read_struct()?;
    assert_lod(&lod, read.prev)
}

fn assert_lod(lod: &LodRcC, offset: usize) -> Result<Lod> {
    // TODO
    // chk!(offset, lod.field00 == 1)?;
    chk!(offset, lod.range_near_sq >= 0.0)?;
    let range_near = lod.range_near_sq.sqrt();
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

    let range = Range {
        min: range_near,
        max: lod.range_far,
    };

    Ok(Lod {
        field00: lod.field00,
        range,
        field16: lod.field16,
        field20: lod.field20,
        field24: lod.field24,
        field28: lod.field28,
        field32: lod.field32,
        field36: lod.field36,
        field40: lod.field40,
        field44: lod.field44,
        field48: lod.field48,
        field52: lod.field52,
        field56: lod.field56,
        field60: lod.field60,
        field64: lod.field64,
        field68: lod.field68,
        field72: lod.field72,
        field76: lod.field76,
    })
}
