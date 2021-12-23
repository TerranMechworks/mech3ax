use super::anim_def::{read_anim_def, read_anim_def_zero, write_anim_def, write_anim_def_zero};
use super::types::{AnimDef, AnimPtr};
use ::serde::{Deserialize, Serialize};
use log::trace;
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, WriteHelper};
use mech3ax_common::serde::base64;
use mech3ax_common::string::{str_from_c_partition, str_to_c_partition};
use mech3ax_common::{assert_that, static_assert_size, Error, Result};
use std::convert::From;
use std::io::{Read, Write};

const SIGNATURE: u32 = 0x08170616;

#[allow(dead_code)]
const VERSION_RECOIL: u32 = 28;
const VERSION_MW: u32 = 39;
#[allow(dead_code)]
const VERSION_PM: u32 = 50;

#[allow(clippy::excessive_precision)]
const GRAVITY: f32 = -9.800000190734863;

#[repr(C)]
struct AnimNameC {
    name: [u8; 80],
    unknown: u32,
}
static_assert_size!(AnimNameC, 84);

#[derive(Debug, Serialize, Deserialize)]
pub struct AnimName {
    pub name: String,
    #[serde(with = "base64")]
    pub pad: Vec<u8>,
    pub unknown: u32,
}

#[repr(C)]
struct AnimInfoC {
    zero00: u32,
    ptr04: u32,
    zero08: u16,
    count: u16,     // 10
    base_ptr: u32,  // 12
    loc_count: u32, // 16
    loc_ptr: u32,   // 20
    world_ptr: u32, // 24
    gravity: f32,   // 28
    zero32: u32,
    zero36: u32,
    zero40: u32,
    zero44: u32,
    zero48: u32,
    zero52: u32,
    zero56: u32,
    one60: u32,
    zero64: u32,
}
static_assert_size!(AnimInfoC, 68);

#[derive(Debug, Serialize, Deserialize)]
pub struct AnimMetadata {
    pub base_ptr: u32,
    pub world_ptr: u32,
    pub anim_names: Vec<AnimName>,
    pub anim_ptrs: Vec<AnimPtr>,
}

fn read_anim_header<R: Read>(read: &mut CountingReader<R>) -> Result<Vec<AnimName>> {
    trace!("Reading anim header at {}", read.offset);
    let signature = read.read_u32()?;
    assert_that!("signature", signature == SIGNATURE, read.prev)?;
    let version = read.read_u32()?;
    assert_that!("version", version == VERSION_MW, read.prev)?;
    let count = read.read_u32()?;

    trace!("Reading anim names at {}", read.offset);
    (0..count)
        .map(|_| {
            let anim_name: AnimNameC = read.read_struct()?;
            let (name, pad) = assert_utf8("anim header name", read.prev + 0, || {
                str_from_c_partition(&anim_name.name)
            })?;
            Ok(AnimName {
                name,
                pad,
                unknown: anim_name.unknown,
            })
        })
        .collect::<Result<Vec<_>>>()
}

fn read_anim_info<R: Read>(read: &mut CountingReader<R>) -> Result<(u16, u32, u32)> {
    trace!("Reading anim info at {}", read.offset);
    let anim_info: AnimInfoC = read.read_struct()?;
    assert_that!("anim field 00", anim_info.zero00 == 0, read.prev + 0)?;
    assert_that!("anim field 04", anim_info.ptr04 == 0, read.prev + 4)?;
    assert_that!("anim field 08", anim_info.zero00 == 0, read.prev + 8)?;
    assert_that!("anim count", anim_info.count > 0, read.prev + 10)?;
    assert_that!("anim base pointer", anim_info.base_ptr != 0, read.prev + 12)?;
    // the localisation isn't used
    assert_that!(
        "anim localisation count",
        anim_info.loc_count == 0,
        read.prev + 16
    )?;
    assert_that!(
        "anim localisation pointer",
        anim_info.loc_ptr == 0,
        read.prev + 20
    )?;
    assert_that!(
        "anim localisation count",
        anim_info.loc_count == 0,
        read.prev + 16
    )?;
    assert_that!(
        "anim world pointer",
        anim_info.world_ptr != 0,
        read.prev + 20
    )?;
    // the gravity is always the same
    assert_that!("anim gravity", anim_info.gravity == GRAVITY, read.prev + 24)?;
    assert_that!("anim field 32", anim_info.zero32 == 0, read.prev + 32)?;
    assert_that!("anim field 36", anim_info.zero36 == 0, read.prev + 36)?;
    assert_that!("anim field 40", anim_info.zero40 == 0, read.prev + 40)?;
    assert_that!("anim field 44", anim_info.zero44 == 0, read.prev + 44)?;
    assert_that!("anim field 48", anim_info.zero48 == 0, read.prev + 48)?;
    assert_that!("anim field 52", anim_info.zero52 == 0, read.prev + 52)?;
    assert_that!("anim field 56", anim_info.zero56 == 0, read.prev + 56)?;
    assert_that!("anim field 60", anim_info.one60 == 1, read.prev + 60)?;
    assert_that!("anim field 64", anim_info.zero64 == 0, read.prev + 64)?;

    Ok((anim_info.count, anim_info.base_ptr, anim_info.world_ptr))
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
    trace!("Reading anim def 0 at {}", read.offset);
    read_anim_def_zero(read)?;
    (1..count)
        .map(|i| {
            trace!("Reading anim def {} at {}", i, read.offset);
            let (anim_def, anim_ptr) = read_anim_def(read)?;
            save_anim_def(&anim_ptr.file_name, &anim_def)?;
            Ok(anim_ptr)
        })
        .collect()
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
    let anim_names = read_anim_header(read)?;
    let (count, base_ptr, world_ptr) = read_anim_info(read)?;
    let anim_ptrs = read_anim_defs(read, count, save_anim_def)?;
    read.assert_end()?;
    Ok(AnimMetadata {
        base_ptr,
        world_ptr,
        anim_names,
        anim_ptrs,
    })
}

fn write_anim_header<W: Write>(write: &mut W, anim_names: &[AnimName]) -> Result<()> {
    write.write_u32(SIGNATURE)?;
    write.write_u32(VERSION_MW)?;
    write.write_u32(anim_names.len() as u32)?;

    for anim_name in anim_names {
        let mut name = [0; 80];
        str_to_c_partition(&anim_name.name, &anim_name.pad, &mut name);
        write.write_struct(&AnimNameC {
            name,
            unknown: anim_name.unknown,
        })?;
    }
    Ok(())
}

fn write_anim_info<W: Write>(write: &mut W, metadata: &AnimMetadata) -> Result<()> {
    write.write_struct(&AnimInfoC {
        zero00: 0,
        ptr04: 0,
        zero08: 0,
        count: metadata.anim_ptrs.len() as u16 + 1,
        base_ptr: metadata.base_ptr,
        loc_count: 0,
        loc_ptr: 0,
        world_ptr: metadata.world_ptr,
        gravity: GRAVITY,
        zero32: 0,
        zero36: 0,
        zero40: 0,
        zero44: 0,
        zero48: 0,
        zero52: 0,
        zero56: 0,
        one60: 1,
        zero64: 0,
    })?;
    Ok(())
}

fn write_anim_defs<W, F, E>(
    write: &mut W,
    anim_ptrs: &[AnimPtr],
    mut load_anim_def: F,
) -> std::result::Result<(), E>
where
    W: Write,
    F: FnMut(&str) -> std::result::Result<AnimDef, E>,
    E: From<std::io::Error> + From<Error>,
{
    write_anim_def_zero(write)?;
    for anim_ptr in anim_ptrs {
        let anim_def = load_anim_def(&anim_ptr.file_name)?;
        write_anim_def(write, &anim_def, anim_ptr)?;
    }
    Ok(())
}

pub fn write_anim<W, F, E>(
    write: &mut W,
    metadata: &AnimMetadata,
    load_anim_def: F,
) -> std::result::Result<(), E>
where
    W: Write,
    F: FnMut(&str) -> std::result::Result<AnimDef, E>,
    E: From<std::io::Error> + From<Error>,
{
    write_anim_header(write, &metadata.anim_names)?;
    write_anim_info(write, metadata)?;
    write_anim_defs(write, &metadata.anim_ptrs, load_anim_def)?;
    Ok(())
}
