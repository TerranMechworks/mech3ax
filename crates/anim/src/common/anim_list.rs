use crate::common::fixup::{Fwd, Rev};
use bytemuck::{AnyBitPattern, NoUninit};
use log::trace;
use mech3ax_api_types::anim::AnimDefFile;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::Result;
use mech3ax_timestamp::unix::{from_timestamp, to_timestamp};
use mech3ax_types::{impl_as_bytes, Ascii};
use std::io::{Read, Write};

/// An `ANIMATION_DEFINITION_FILE` in an `ANIMATION_LIST`.
#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct AnimDefFileC {
    name: Ascii<80>,
    timestamp: u32,
}
impl_as_bytes!(AnimDefFileC, 84);

/// Read an `ANIMATION_LIST`.
pub(crate) fn read_anim_list<F>(
    read: &mut CountingReader<impl Read>,
    count: u32,
    fwd: F,
) -> Result<Vec<AnimDefFile>>
where
    F: Fn(&[u8; 80]) -> Option<(u32, &'static str)>,
{
    trace!("Reading animation list ({})", count);
    let fwd = Fwd::new("anim def file name", fwd);
    (0..count)
        .map(|_| {
            let anim_def_file: AnimDefFileC = read.read_struct()?;
            let (name, hash) = fwd.fixup(read.prev + 0, &anim_def_file.name)?;
            let datetime = from_timestamp(anim_def_file.timestamp);
            Ok(AnimDefFile {
                name,
                datetime,
                hash,
            })
        })
        .collect()
}

/// Write a `ANIMATION_LIST`.
pub(crate) fn write_anim_list<F>(
    write: &mut CountingWriter<impl Write>,
    anim_list: &[AnimDefFile],
    rev: F,
) -> Result<()>
where
    F: Fn(u32, &str) -> Option<&'static [u8; 80]>,
{
    trace!("Writing animation list ({})", anim_list.len());
    let rev = Rev::new("anim def file name", rev);
    for anim_def_file in anim_list {
        let name = rev.fixup(&anim_def_file.name, anim_def_file.hash);
        let timestamp = to_timestamp(&anim_def_file.datetime);
        let anim_def_file = AnimDefFileC { name, timestamp };
        write.write_struct(&anim_def_file)?;
    }
    Ok(())
}
