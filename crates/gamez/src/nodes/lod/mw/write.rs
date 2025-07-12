use super::LodMwC;
use mech3ax_api_types::gamez::nodes::Lod;
use mech3ax_common::Result;
use mech3ax_common::io_ext::CountingWriter;
use std::io::Write;

pub(crate) fn write(write: &mut CountingWriter<impl Write>, lod: &Lod) -> Result<()> {
    let range_near_sq = lod.range.min * lod.range.min;
    let range_far = lod.range.max;
    let range_far_sq = range_far * range_far;

    let lod = LodMwC {
        field00: lod.field00,
        range_near_sq,
        range_far,
        range_far_sq,
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
    };
    write.write_struct(&lod)?;
    Ok(())
}
