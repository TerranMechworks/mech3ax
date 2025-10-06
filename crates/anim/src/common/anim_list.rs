use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::AnimDefFile;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{Result, assert_len, chk};
use mech3ax_timestamp::unix::{from_timestamp, to_timestamp};
use mech3ax_types::check::{garbage, make_garbage};
use mech3ax_types::{Ascii, Offsets, impl_as_bytes};
use std::io::{Read, Write};

/// An `ANIMATION_LIST` (header?).
#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct AnimListC {
    count: u32,
}
impl_as_bytes!(AnimListC, 4);

/// An `ANIMATION_DEFINITION_FILE` in an `ANIMATION_LIST`.
#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct AnimDefFileC {
    name: Ascii<80>,
    timestamp: u32,
}
impl_as_bytes!(AnimDefFileC, 84);

/// Read an `ANIMATION_LIST`.
pub(crate) fn read_anim_list(read: &mut CountingReader<impl Read>) -> Result<Vec<AnimDefFile>> {
    let AnimListC { count } = read.read_struct()?;
    (0..count)
        .map(|_| {
            let anim_def_file: AnimDefFileC = read.read_struct()?;
            let (name, garbage) = chk!(read.prev, garbage(&anim_def_file.name))?;
            let datetime = from_timestamp(anim_def_file.timestamp);
            Ok(AnimDefFile {
                name,
                datetime,
                garbage,
            })
        })
        .collect()
}

/// Write a `ANIMATION_LIST`.
pub(crate) fn write_anim_list(
    write: &mut CountingWriter<impl Write>,
    anim_list: &[AnimDefFile],
) -> Result<()> {
    let count = assert_len!(u32, anim_list.len(), "anim list")?;
    write.write_struct(&AnimListC { count })?;
    for anim_def_file in anim_list {
        let name = make_garbage(&anim_def_file.name, &anim_def_file.garbage);
        let timestamp = to_timestamp(&anim_def_file.datetime);
        let anim_def_file = AnimDefFileC { name, timestamp };
        write.write_struct(&anim_def_file)?;
    }
    Ok(())
}
