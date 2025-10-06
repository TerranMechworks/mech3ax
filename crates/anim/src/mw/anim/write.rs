use super::{AnimHeaderC, AnimInfoC, Mission};
use crate::common::anim_list::write_anim_list;
use crate::common::si_script::load_anim_scripts;
use crate::mw::anim_def::{write_anim_def, write_anim_def_zero};
use crate::{LoadItem, LoadItemName, SIGNATURE, VERSION_MW};
use log::{debug, trace};
use mech3ax_api_types::anim::{AnimMetadata, SiScript};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{Error, Result, assert_len};
use mech3ax_types::EnumerateEx as _;
use std::convert::From;
use std::io::Write;

pub fn write_anim<W, F, E>(
    write: &mut CountingWriter<W>,
    metadata: &AnimMetadata,
    mut load_item: F,
) -> std::result::Result<(), E>
where
    W: Write,
    F: FnMut(LoadItemName<'_>) -> std::result::Result<LoadItem, E>,
    E: From<std::io::Error> + From<Error>,
{
    write_anim_header(write)?;
    write_anim_list(write, &metadata.anim_list)?;
    write_anim_info(write, metadata)?;
    let scripts = load_anim_scripts(&metadata.script_names, &mut load_item)?;
    write_anim_defs(write, &metadata.anim_def_names, load_item, &scripts)?;
    Ok(())
}

fn write_anim_header(write: &mut CountingWriter<impl Write>) -> Result<()> {
    let header = AnimHeaderC {
        signature: SIGNATURE,
        version: VERSION_MW,
    };
    write.write_struct(&header)?;
    Ok(())
}

fn write_anim_info(write: &mut CountingWriter<impl Write>, metadata: &AnimMetadata) -> Result<()> {
    let m = Mission::from_api(metadata.mission);

    let def_count = assert_len!(u16, metadata.anim_def_names.len() + 1, "anim defs")?;

    let anim_info = AnimInfoC {
        zero00: 0,
        zero04: 0,
        zero08: 0,
        def_count,
        defs_ptr: m.defs_ptr(),
        msg_count: 0,
        msgs_ptr: 0,
        world_ptr: m.world_ptr(),
        gravity: metadata.gravity,
        zero32: 0,
        zero36: 0,
        zero40: 0,
        zero44: 0,
        zero48: 0,
        zero52: 0,
        zero56: 0,
        one60: 1,
        zero64: 0,
    };
    write.write_struct(&anim_info)?;
    Ok(())
}

fn write_anim_defs<W, F, E>(
    write: &mut CountingWriter<W>,
    anim_def_names: &[String],
    mut load_item: F,
    scripts: &[SiScript],
) -> std::result::Result<(), E>
where
    W: Write,
    F: FnMut(LoadItemName<'_>) -> std::result::Result<LoadItem, E>,
    E: From<std::io::Error> + From<Error>,
{
    trace!("Writing anim def 0");
    write_anim_def_zero(write)?;
    for (index, anim_def_name) in anim_def_names.iter().enumerate_one() {
        debug!("Loading anim def {}: `{}`", index, anim_def_name);
        let item_name = LoadItemName::AnimDef(anim_def_name);
        let anim_def = load_item(item_name)?.anim_def(anim_def_name)?;

        trace!("Writing anim def {}", index);
        write_anim_def(write, &anim_def, scripts)?;
    }
    Ok(())
}
