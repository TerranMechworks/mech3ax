use super::{AnimHeaderC, AnimInfoC, Mission};
use crate::common::anim_list::read_anim_list;
use crate::common::si_script::save_anim_scripts;
use crate::mw::anim_def::{read_anim_def, read_anim_def_zero};
use crate::{SaveItem, SIGNATURE, VERSION_MW};
use log::{debug, trace};
use mech3ax_anim_names::mw::anim_list_fwd;
use mech3ax_api_types::anim::{AnimDefName, AnimMetadata, SiScript};
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{assert_that, Error, Result};
use std::convert::From;
use std::io::Read;

#[derive(Debug)]
struct AnimInfo {
    m: Mission,
    gravity: f32,
    def_count: u16,
}

pub fn read_anim<R, F, E>(
    read: &mut CountingReader<R>,
    mut save_item: F,
) -> std::result::Result<AnimMetadata, E>
where
    R: Read,
    F: FnMut(SaveItem<'_>) -> std::result::Result<(), E>,
    E: From<std::io::Error> + From<Error>,
{
    read_anim_header(read)?;
    let anim_list = read_anim_list(read, anim_list_fwd)?;
    let anim_info = read_anim_info(read)?;
    let mut scripts = Vec::new();
    let anim_def_names = read_anim_defs(read, anim_info.def_count, &mut save_item, &mut scripts)?;
    read.assert_end()?;
    let script_names = save_anim_scripts(scripts, save_item)?;

    Ok(AnimMetadata {
        mission: anim_info.m.to_api(),
        gravity: anim_info.gravity,
        datetime: None,
        script_names,
        anim_def_names,
        anim_list,
    })
}

fn read_anim_header(read: &mut CountingReader<impl Read>) -> Result<()> {
    let header: AnimHeaderC = read.read_struct()?;
    assert_that!("signature", header.signature == SIGNATURE, read.prev)?;
    assert_that!("version", header.version == VERSION_MW, read.prev)?;
    Ok(())
}

fn read_anim_info(read: &mut CountingReader<impl Read>) -> Result<AnimInfo> {
    let anim_info: AnimInfoC = read.read_struct()?;

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
    let m = Mission::from_defs_ptr(anim_info.defs_ptr);

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

    assert_that!("anim info field 000", anim_info.zero00 == 0, read.prev + 0)?;
    assert_that!("anim info field 004", anim_info.zero04 == 0, read.prev + 4)?;
    assert_that!("anim info field 008", anim_info.zero08 == 0, read.prev + 8)?;

    assert_that!("anim info field 032", anim_info.zero32 == 0, read.prev + 32)?;
    assert_that!("anim info field 036", anim_info.zero36 == 0, read.prev + 36)?;
    assert_that!("anim info field 040", anim_info.zero40 == 0, read.prev + 40)?;
    assert_that!("anim info field 044", anim_info.zero44 == 0, read.prev + 44)?;
    assert_that!("anim info field 048", anim_info.zero48 == 0, read.prev + 48)?;
    assert_that!("anim info field 052", anim_info.zero52 == 0, read.prev + 52)?;
    assert_that!("anim info field 056", anim_info.zero56 == 0, read.prev + 56)?;
    assert_that!("anim info field 060", anim_info.one60 == 1, read.prev + 60)?;
    assert_that!("anim info field 064", anim_info.zero64 == 0, read.prev + 64)?;

    Ok(AnimInfo {
        m,
        gravity: anim_info.gravity,
        def_count: anim_info.def_count,
    })
}

fn read_anim_defs<R, F, E>(
    read: &mut CountingReader<R>,
    count: u16,
    mut save_item: F,
    scripts: &mut Vec<SiScript>,
) -> std::result::Result<Vec<AnimDefName>, E>
where
    R: Read,
    F: FnMut(SaveItem<'_>) -> std::result::Result<(), E>,
    E: From<std::io::Error> + From<Error>,
{
    trace!("Reading anim def 0");
    read_anim_def_zero(read)?;
    (1..count)
        .map(|index| {
            trace!("Reading anim def {}", index);
            let (anim_def, anim_def_name) = read_anim_def(read, scripts)?;

            debug!("Saving anim def {}: `{}`", index, anim_def_name.file_name);
            let item = SaveItem::AnimDef {
                name: &anim_def_name.file_name,
                anim_def: &anim_def,
            };
            save_item(item)?;
            Ok(anim_def_name)
        })
        .collect()
}
