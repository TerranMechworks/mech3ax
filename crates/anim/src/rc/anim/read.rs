use super::{AnimHeaderC, AnimInfoC};
use crate::common::anim_list::read_anim_list;
use crate::rc::anim_def::{read_anim_def, read_anim_def_zero};
use crate::{GRAVITY, SIGNATURE, VERSION_RC};
use log::{debug, trace};
use mech3ax_anim_names::rc::anim_list_fwd;
use mech3ax_api_types::anim::{AnimDef, AnimMetadata, AnimPtr, SiScript};
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{assert_that, Error, Rename, Result};
use std::convert::From;
use std::io::Read;

pub fn read_anim<R, F, E>(
    read: &mut CountingReader<R>,
    save_anim_def: F,
) -> std::result::Result<AnimMetadata, E>
where
    R: Read,
    F: FnMut(&str, &AnimDef) -> std::result::Result<(), E>,
    E: From<std::io::Error> + From<Error>,
{
    read_anim_header(read)?;
    let anim_list = read_anim_list(read, anim_list_fwd)?;
    let (count, defs_ptr, world_ptr) = read_anim_info(read)?;
    let mut scripts = Vec::new();
    let anim_ptrs = read_anim_defs(read, count, save_anim_def, &mut scripts)?;
    read.assert_end()?;

    Ok(AnimMetadata {
        defs_ptr,
        world_ptr,
        anim_list,
        anim_ptrs,
        scripts,
        datetime: None,
        unk40: 0,
        scripts_ptr: 0,
    })
}

fn read_anim_header(read: &mut CountingReader<impl Read>) -> Result<()> {
    let header: AnimHeaderC = read.read_struct()?;
    assert_that!("signature", header.signature == SIGNATURE, read.prev)?;
    assert_that!("version", header.version == VERSION_RC, read.prev)?;
    Ok(())
}

fn read_anim_info(read: &mut CountingReader<impl Read>) -> Result<(u16, u32, u32)> {
    let anim_info: AnimInfoC = read.read_struct()?;

    assert_that!("anim info field 00", anim_info.zero00 == 0, read.prev + 0)?;
    assert_that!("anim info field 04", anim_info.zero04 == 0, read.prev + 4)?;
    assert_that!("anim info field 08", anim_info.zero00 == 0, read.prev + 8)?;
    assert_that!(
        "anim info def count",
        anim_info.def_count > 0,
        read.prev + 10
    )?;
    assert_that!(
        "anim info defs pointer",
        anim_info.defs_ptr != 0,
        read.prev + 12
    )?;
    // the messages/localisation aren't used
    assert_that!(
        "anim info msg count",
        anim_info.msg_count == 0,
        read.prev + 16
    )?;
    assert_that!(
        "anim info msgs pointer",
        anim_info.msgs_ptr == 0,
        read.prev + 20
    )?;
    assert_that!(
        "anim info world pointer",
        anim_info.world_ptr != 0,
        read.prev + 24
    )?;
    // the gravity is always the same
    assert_that!(
        "anim info gravity",
        anim_info.gravity == GRAVITY,
        read.prev + 28
    )?;
    assert_that!("anim info field 32", anim_info.zero32 == 0, read.prev + 32)?;
    assert_that!("anim info field 36", anim_info.zero36 == 0, read.prev + 36)?;
    assert_that!("anim info field 40", anim_info.zero40 == 0, read.prev + 40)?;
    assert_that!("anim info field 44", anim_info.zero44 == 0, read.prev + 44)?;
    assert_that!("anim info field 48", anim_info.zero48 == 0, read.prev + 48)?;
    assert_that!("anim info field 52", anim_info.zero52 == 0, read.prev + 52)?;
    assert_that!("anim info field 56", anim_info.zero56 == 0, read.prev + 56)?;

    Ok((anim_info.def_count, anim_info.defs_ptr, anim_info.world_ptr))
}

fn read_anim_defs<R, F, E>(
    read: &mut CountingReader<R>,
    count: u16,
    mut save_anim_def: F,
    scripts: &mut Vec<SiScript>,
) -> std::result::Result<Vec<AnimPtr>, E>
where
    R: Read,
    F: FnMut(&str, &AnimDef) -> std::result::Result<(), E>,
    E: From<std::io::Error> + From<Error>,
{
    trace!("Reading anim def 0");
    read_anim_def_zero(read)?;

    let mut seen = Rename::new();
    (1..count)
        .map(|index| {
            trace!("Reading anim def {}", index);
            let (anim_def, mut anim_ptr) = read_anim_def(read, scripts)?;

            anim_ptr.rename = seen.insert(&anim_ptr.file_name);
            let file_name = anim_ptr
                .rename
                .as_deref()
                .inspect(|rename| {
                    debug!("Renaming anim def `{}` to `{}`", anim_ptr.file_name, rename)
                })
                .unwrap_or(&anim_ptr.file_name);

            debug!("Saving anim def {}: `{}`", index, file_name);
            save_anim_def(file_name, &anim_def)?;
            Ok(anim_ptr)
        })
        .collect()
}
