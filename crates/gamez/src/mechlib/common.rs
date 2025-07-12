use crate::materials::{read_material_mechlib, write_material_mechlib};
use log::trace;
use mech3ax_api_types::Count32;
use mech3ax_api_types::gamez::MechlibMaterial;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{GameType, Result, chk, err, len};
use std::io::{Read, Write};

pub const VERSION_MW: u32 = 27;
pub const VERSION_PM: u32 = 41;
pub const FORMAT: u32 = 1;

pub fn read_version(read: &mut CountingReader<impl Read>, game: GameType) -> Result<()> {
    let version = read.read_u32()?;
    let expected = match game {
        GameType::MW => VERSION_MW,
        GameType::PM => VERSION_PM,
        GameType::RC => return Err(err!("Recoil has no mechlib")),
        GameType::CS => return Err(err!("Crimson Skies has no mechlib")),
    };
    chk!(read.prev, version == expected)?;
    read.assert_end()
}

pub fn read_format(read: &mut CountingReader<impl Read>) -> Result<()> {
    let format = read.read_u32()?;
    chk!(read.prev, format == FORMAT)?;
    read.assert_end()
}

pub fn write_version(write: &mut CountingWriter<impl Write>, game: GameType) -> Result<()> {
    let version = match game {
        GameType::MW => VERSION_MW,
        GameType::PM => VERSION_PM,
        GameType::RC => return Err(err!("Recoil has no mechlib")),
        GameType::CS => return Err(err!("Crimson Skies has no mechlib")),
    };
    write.write_u32(version)?;
    Ok(())
}

pub fn write_format(write: &mut CountingWriter<impl Write>) -> Result<()> {
    write.write_u32(FORMAT)?;
    Ok(())
}

pub fn read_materials(read: &mut CountingReader<impl Read>) -> Result<Vec<MechlibMaterial>> {
    let mechlib_material_count = Count32::new(read.read_i32()?);
    let count = chk!(read.prev, ?mechlib_material_count)?;

    let materials = count
        .iter()
        .map(|index| {
            trace!("Processing material {}/{}", index, count);
            read_material_mechlib(read)
        })
        .collect::<Result<Vec<_>>>()?;

    read.assert_end()?;
    Ok(materials)
}

pub fn write_materials(
    write: &mut CountingWriter<impl Write>,
    materials: &[MechlibMaterial],
) -> Result<()> {
    let count = len!(materials.len(), "Mechlib materials")?;
    write.write_i32(count.to_i32())?;

    for (index, material) in materials.iter().enumerate() {
        trace!("Processing material {}/{}", index, count);
        write_material_mechlib(write, material)?;
    }

    Ok(())
}
