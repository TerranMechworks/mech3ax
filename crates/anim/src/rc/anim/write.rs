use super::{AnimHeaderC, AnimInfoC};
use crate::common::anim_list::write_anim_list;
use crate::rc::anim_def::{write_anim_def, write_anim_def_zero};
use crate::{GRAVITY, SIGNATURE, VERSION_RC};
use log::{debug, trace};
use mech3ax_anim_names::rc::anim_list_rev;
use mech3ax_api_types::anim::{AnimDef, AnimDefFile, AnimMetadata, AnimPtr, SiScript};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{assert_len, Error, Result};
use mech3ax_types::EnumerateEx as _;
use std::convert::From;
use std::io::Write;

pub fn write_anim<W, F, E>(
    write: &mut CountingWriter<W>,
    metadata: &AnimMetadata,
    load_anim_def: F,
) -> std::result::Result<(), E>
where
    W: Write,
    F: FnMut(&str) -> std::result::Result<AnimDef, E>,
    E: From<std::io::Error> + From<Error>,
{
    write_anim_header(write, &metadata.anim_list)?;
    write_anim_info(write, metadata)?;
    write_anim_defs(write, &metadata.anim_ptrs, load_anim_def, &metadata.scripts)?;
    Ok(())
}

fn write_anim_header(
    write: &mut CountingWriter<impl Write>,
    anim_list: &[AnimDefFile],
) -> Result<()> {
    let count = assert_len!(u32, anim_list.len(), "anim list")?;
    let header = AnimHeaderC {
        signature: SIGNATURE,
        version: VERSION_RC,
        count,
    };
    write.write_struct(&header)?;
    write_anim_list(write, anim_list, anim_list_rev)
}

fn write_anim_info(write: &mut CountingWriter<impl Write>, metadata: &AnimMetadata) -> Result<()> {
    let def_count = assert_len!(u16, metadata.anim_ptrs.len() + 1, "anim defs")?;

    let anim_info = AnimInfoC {
        zero00: 0,
        zero04: 0,
        zero08: 0,
        def_count,
        defs_ptr: metadata.defs_ptr,
        loc_count: 0,
        locs_ptr: 0,
        world_ptr: metadata.world_ptr,
        gravity: GRAVITY,
        zero32: 0,
        zero36: 0,
        zero40: 0,
        zero44: 0,
        zero48: 0,
        zero52: 0,
        zero56: 0,
    };
    write.write_struct(&anim_info)?;
    Ok(())
}

fn write_anim_defs<W, F, E>(
    write: &mut CountingWriter<W>,
    anim_ptrs: &[AnimPtr],
    mut load_anim_def: F,
    scripts: &[SiScript],
) -> std::result::Result<(), E>
where
    W: Write,
    F: FnMut(&str) -> std::result::Result<AnimDef, E>,
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
        let anim_def = load_anim_def(file_name)?;

        trace!("Writing anim def {}", index);
        write_anim_def(write, &anim_def, anim_ptr, scripts)?;
    }
    Ok(())
}
