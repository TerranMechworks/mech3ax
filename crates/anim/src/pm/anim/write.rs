use super::{AnimHeaderC, AnimInfoC, Mission, SiScriptC};
use crate::common::anim_list::write_anim_list;
use crate::pm::anim_def::{write_anim_def, write_anim_def_zero};
use crate::{LoadItem, LoadItemName, SIGNATURE, VERSION_PM};
use log::{debug, trace};
use mech3ax_anim_events::si_script::{size_si_script_frames, write_si_script_frames};
use mech3ax_anim_names::pm::anim_list_rev;
use mech3ax_api_types::anim::{AnimMetadata, AnimPtr, SiScript};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{assert_len, assert_with_msg, Error, Result};
use mech3ax_timestamp::unix::to_timestamp;
use mech3ax_timestamp::DateTime;
use mech3ax_types::{str_to_ascii, EnumerateEx as _};
use std::convert::From;
use std::io::Write;

pub fn write_anim<W, F, E>(
    write: &mut CountingWriter<W>,
    metadata: &AnimMetadata,
    load_item: F,
) -> std::result::Result<(), E>
where
    W: Write,
    F: FnMut(LoadItemName<'_>) -> std::result::Result<LoadItem, E>,
    E: From<std::io::Error> + From<Error>,
{
    let datetime = metadata.datetime.as_ref();
    write_anim_header(write, datetime)?;
    write_anim_list(write, &metadata.anim_list, anim_list_rev)?;
    write_anim_info(write, metadata)?;
    write_anim_defs(write, &metadata.anim_ptrs, load_item)?;
    write_anim_scripts(write, &metadata.scripts)?;
    Ok(())
}

fn write_anim_header(
    write: &mut CountingWriter<impl Write>,
    datetime: Option<&DateTime>,
) -> Result<()> {
    if let Some(dt) = datetime {
        trace!("anim datetime: `{:?}`", dt);
    }
    let timestamp = datetime.map(to_timestamp).unwrap_or(0);

    let header = AnimHeaderC {
        signature: SIGNATURE,
        version: VERSION_PM,
        timestamp,
    };
    write.write_struct(&header)?;
    Ok(())
}

fn write_anim_info(write: &mut CountingWriter<impl Write>, metadata: &AnimMetadata) -> Result<()> {
    let m = Mission::from_api(metadata.mission);

    let def_count = assert_len!(u16, metadata.anim_ptrs.len() + 1, "anim defs")?;
    let script_count = assert_len!(u32, metadata.scripts.len(), "anim scripts")?;

    let anim_info = AnimInfoC {
        zero00: 0,
        zero04: 0,
        zero08: 0,
        def_count,
        defs_ptr: m.defs_ptr(),
        script_count,
        scripts_ptr: m.scripts_ptr(),
        msg_count: 0,
        msgs_ptr: 0,
        world_ptr: m.world_ptr(),
        gravity: metadata.gravity,
        unk40: m.unk40(),
        zero44: 0,
        zero48: 0,
        zero52: 0,
        zero56: 0,
        one60: 1,
        zero64: 0,
        zero68: 0,
        zero72: 0,
        zero76: 0,
        zero80: 0,
        zero84: 0,
        zero88: 0,
        zero92: 0,
        zero96: 0,
        zero100: 0,
        zero104: 0,
    };
    write.write_struct(&anim_info)?;
    Ok(())
}

fn write_anim_defs<W, F, E>(
    write: &mut CountingWriter<W>,
    anim_ptrs: &[AnimPtr],
    mut load_item: F,
) -> std::result::Result<(), E>
where
    W: Write,
    F: FnMut(LoadItemName<'_>) -> std::result::Result<LoadItem, E>,
    E: From<std::io::Error> + From<Error>,
{
    trace!("Writing anim def 0");
    write_anim_def_zero(write)?;
    for (index, anim_ptr) in anim_ptrs.iter().enumerate_one() {
        let file_name = anim_ptr
            .rename
            .as_deref()
            .inspect(|rename| debug!("Renaming anim def `{}` to `{}`", anim_ptr.file_name, rename))
            .unwrap_or(&anim_ptr.file_name);

        debug!("Loading anim def {}: `{}`", index, file_name);
        let item_name = LoadItemName::AnimDef(file_name);
        let anim_def = load_item(item_name)?.anim_def(file_name)?;

        trace!("Writing anim def {}", index);
        write_anim_def(write, &anim_def, anim_ptr)?;
    }
    Ok(())
}

fn write_str_zero_terminated(
    write: &mut CountingWriter<impl Write>,
    name: &str,
    buf: &mut [u8; 256],
    s: &str,
    index: usize,
) -> Result<()> {
    let src = assert_utf8(name, index, || str_to_ascii(s))?;

    let len = src.len() + 1;
    let slice = &mut buf[..len];
    let (end, dst) = slice.split_last_mut().expect("empty string");
    dst.copy_from_slice(src);
    *end = 0;

    trace!(
        "`{}` (len: {}, at {})",
        slice.escape_ascii(),
        len,
        write.offset
    );
    write.write_all(slice)?;
    Ok(())
}

fn write_anim_scripts(write: &mut CountingWriter<impl Write>, scripts: &[SiScript]) -> Result<()> {
    for (index, script) in scripts.iter().enumerate() {
        trace!("Writing anim script info {}", index);

        let script_name_len = assert_len!(
            u8,
            script.script_name.len() + 1,
            "si script script name len"
        )?;
        let object_name_len = assert_len!(
            u8,
            script.object_name.len() + 1,
            "si script object name len"
        )?;
        let spline_interp = script.spline_interp.into();
        let frame_count = assert_len!(u32, script.frames.len(), "si script frame len")?;
        let script_data_len = size_si_script_frames(&script.frames)
            .ok_or_else(|| assert_with_msg!("Anim script {} frame size overflow", index))?;

        let si = SiScriptC {
            script_name_ptr: script.script_name_ptr,
            object_name_ptr: script.object_name_ptr,
            script_name_len,
            object_name_len,
            pad10: 0,
            spline_interp,
            frame_count,
            script_data_len,
            script_data_ptr: script.script_data_ptr,
        };
        write.write_struct(&si)?;
    }

    let mut buf = [0u8; 256];
    for (index, script) in scripts.iter().enumerate() {
        trace!("Writing anim script data {}", index);

        write_str_zero_terminated(
            write,
            "si script script name",
            &mut buf,
            &script.script_name,
            index,
        )?;
        write_str_zero_terminated(
            write,
            "si script object name",
            &mut buf,
            &script.object_name,
            index,
        )?;
        write_si_script_frames(write, &script.frames)?;
    }
    Ok(())
}
