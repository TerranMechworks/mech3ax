use super::{AnimHeaderC, AnimInfoC, SiScriptC};
use crate::common::anim_list::read_anim_list;
use crate::pm::anim_def::{read_anim_def, read_anim_def_zero};
use crate::{GRAVITY, SIGNATURE, VERSION_PM};
use log::{debug, trace};
use mech3ax_anim_events::si_script::read_si_script_frames;
use mech3ax_anim_names::pm::anim_list_fwd;
use mech3ax_api_types::anim::{AnimDef, AnimMetadata, AnimPtr, SiScript};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{assert_that, Error, Rename, Result};
use mech3ax_timestamp::unix::from_timestamp;
use mech3ax_timestamp::DateTime;
use mech3ax_types::{str_from_ascii, u32_to_usize, ConversionError};
use std::convert::From;
use std::io::Read;

#[derive(Debug)]
struct AnimInfo {
    def_count: u16,
    defs_ptr: u32,
    script_count: u32,
    scripts_ptr: u32,
    world_ptr: u32,
    unk40: u32,
}

pub fn read_anim<R, F, E>(
    read: &mut CountingReader<R>,
    save_anim_def: F,
) -> std::result::Result<AnimMetadata, E>
where
    R: Read,
    F: FnMut(&str, &AnimDef) -> std::result::Result<(), E>,
    E: From<std::io::Error> + From<Error>,
{
    let datetime = read_anim_header(read)?;
    let anim_list = read_anim_list(read, anim_list_fwd)?;
    let anim_info = read_anim_info(read)?;
    let anim_ptrs = read_anim_defs(read, anim_info.def_count, save_anim_def)?;
    let scripts = read_anim_scripts(read, anim_info.script_count)?;
    read.assert_end()?;

    Ok(AnimMetadata {
        anim_list,
        anim_ptrs,
        scripts,
        datetime: Some(datetime),
        defs_ptr: anim_info.defs_ptr,
        scripts_ptr: anim_info.scripts_ptr,
        world_ptr: anim_info.world_ptr,
        unk40: anim_info.unk40,
    })
}

fn read_anim_header(read: &mut CountingReader<impl Read>) -> Result<DateTime> {
    let header: AnimHeaderC = read.read_struct()?;

    assert_that!("signature", header.signature == SIGNATURE, read.prev)?;
    assert_that!("version", header.version == VERSION_PM, read.prev)?;
    let datetime = from_timestamp(header.timestamp);
    trace!("anim datetime: `{:?}`", datetime);

    Ok(datetime)
}

fn read_anim_info(read: &mut CountingReader<impl Read>) -> Result<AnimInfo> {
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
    assert_that!(
        "anim info script count",
        anim_info.script_count > 0,
        read.prev + 16
    )?;
    assert_that!(
        "anim info script pointer",
        anim_info.scripts_ptr != 0,
        read.prev + 20
    )?;
    // the messages/localisation aren't used
    assert_that!(
        "anim info msg count",
        anim_info.msg_count == 0,
        read.prev + 24
    )?;
    assert_that!(
        "anim info msgs pointer",
        anim_info.msgs_ptr == 0,
        read.prev + 28
    )?;
    assert_that!(
        "anim info world pointer",
        anim_info.world_ptr != 0,
        read.prev + 32
    )?;
    // the gravity is always the same
    assert_that!(
        "anim info gravity",
        anim_info.gravity == GRAVITY,
        read.prev + 36
    )?;
    assert_that!("anim info field 40", anim_info.unk40 in [0, 1], read.prev + 40)?;
    assert_that!("anim info field 44", anim_info.zero44 == 0, read.prev + 44)?;
    assert_that!("anim info field 48", anim_info.zero48 == 0, read.prev + 48)?;
    assert_that!("anim info field 52", anim_info.zero52 == 0, read.prev + 52)?;
    assert_that!("anim info field 56", anim_info.zero56 == 0, read.prev + 56)?;
    assert_that!("anim info field 60", anim_info.one60 == 1, read.prev + 60)?;
    assert_that!("anim info field 64", anim_info.zero64 == 0, read.prev + 64)?;
    assert_that!("anim info field 68", anim_info.zero68 == 0, read.prev + 68)?;
    assert_that!("anim info field 72", anim_info.zero72 == 0, read.prev + 72)?;
    assert_that!("anim info field 76", anim_info.zero76 == 0, read.prev + 76)?;
    assert_that!("anim info field 80", anim_info.zero80 == 0, read.prev + 80)?;
    assert_that!("anim info field 84", anim_info.zero84 == 0, read.prev + 84)?;
    assert_that!("anim info field 88", anim_info.zero88 == 0, read.prev + 88)?;
    assert_that!("anim info field 92", anim_info.zero92 == 0, read.prev + 92)?;
    assert_that!("anim info field 96", anim_info.zero96 == 0, read.prev + 96)?;
    assert_that!(
        "anim info field 100",
        anim_info.zero100 == 0,
        read.prev + 100
    )?;
    assert_that!(
        "anim info field 104",
        anim_info.zero104 == 0,
        read.prev + 104
    )?;

    Ok(AnimInfo {
        def_count: anim_info.def_count,
        defs_ptr: anim_info.defs_ptr,
        script_count: anim_info.script_count,
        scripts_ptr: anim_info.scripts_ptr,
        world_ptr: anim_info.world_ptr,
        unk40: anim_info.unk40,
    })
}

fn read_anim_defs<R, F, E>(
    read: &mut CountingReader<R>,
    count: u16,
    mut save_anim_def: F,
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
            let (anim_def, mut anim_ptr) = read_anim_def(read)?;

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

fn str_from_zero_terminated(buf: &[u8]) -> std::result::Result<String, ConversionError> {
    let (end, v) = buf.split_last().expect("empty string");
    if *end != 0 {
        return Err(ConversionError::Unterminated);
    }
    str_from_ascii(v).map(str::to_string)
}

fn read_str_zero_terminated(
    read: &mut CountingReader<impl Read>,
    name: &str,
    buf: &mut [u8; 256],
    len: u8,
) -> Result<String> {
    let len: usize = len.into();
    let slice = &mut buf[..len];
    read.read_exact(slice)?;
    trace!(
        "`{}` (len: {}, at {})",
        slice.escape_ascii(),
        len,
        read.prev
    );
    Ok(assert_utf8(name, read.prev, || {
        str_from_zero_terminated(slice)
    })?)
}

fn read_anim_scripts(read: &mut CountingReader<impl Read>, count: u32) -> Result<Vec<SiScript>> {
    let scripts = (0..count)
        .map(|index| {
            trace!("Reading anim script info {}", index);
            let si: SiScriptC = read.read_struct()?;

            assert_that!(
                "si script script name ptr",
                si.script_name_ptr != 0,
                read.prev + 0
            )?;
            assert_that!(
                "si script object name ptr",
                si.object_name_ptr != 0,
                read.prev + 4
            )?;
            assert_that!(
                "si script script name len",
                si.script_name_len > 0,
                read.prev + 8
            )?;
            assert_that!(
                "si script object name len",
                si.object_name_len > 0,
                read.prev + 9
            )?;
            assert_that!("si script field 10", si.pad10 == 0, read.prev + 10)?;
            // this is never set in PM
            let spline_interp = assert_that!(
                "si script spline interp",
                bool si.spline_interp,
                read.prev + 12
            )?;
            assert_that!("si script frame count", si.frame_count > 0, read.prev + 16)?;
            assert_that!(
                "si script script data len",
                si.script_data_len > 0,
                read.prev + 20
            )?;
            assert_that!(
                "si script object data ptr",
                si.script_data_ptr != 0,
                read.prev + 24
            )?;
            Ok((si, spline_interp))
        })
        .collect::<Result<Vec<_>>>()?;

    let mut buf = [0u8; 256];
    scripts
        .into_iter()
        .enumerate()
        .map(|(index, (si, spline_interp))| {
            trace!("Reading anim script data {}", index);

            let script_name = read_str_zero_terminated(
                read,
                "si script script name",
                &mut buf,
                si.script_name_len,
            )?;
            let object_name = read_str_zero_terminated(
                read,
                "si script object name",
                &mut buf,
                si.object_name_len,
            )?;
            let size = u32_to_usize(si.script_data_len);
            let frames = read_si_script_frames(read, size, si.frame_count)?;

            Ok(SiScript {
                script_name,
                object_name,
                frames,
                spline_interp,
                script_name_ptr: si.script_name_ptr,
                object_name_ptr: si.object_name_ptr,
                script_data_ptr: si.script_data_ptr,
            })
        })
        .collect::<Result<Vec<_>>>()
}
