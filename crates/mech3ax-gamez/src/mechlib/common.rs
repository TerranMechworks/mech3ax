use crate::materials::ng::{read_material, write_material, RawMaterial};
use mech3ax_api_types::gamez::{Material, TexturedMaterial};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, assert_with_msg, GameType, Result};
use std::io::{Read, Write};

pub const VERSION_MW: u32 = 27;
pub const VERSION_PM: u32 = 41;
pub const FORMAT: u32 = 1;

pub fn read_version(read: &mut CountingReader<impl Read>, game: GameType) -> Result<()> {
    let actual = read.read_u32()?;
    let expected = match game {
        GameType::MW => VERSION_MW,
        GameType::PM => VERSION_PM,
        GameType::RC => return Err(assert_with_msg!("Recoil has no mechlib")),
        GameType::CS => return Err(assert_with_msg!("Crimson Skies has no mechlib")),
    };
    assert_that!("version", actual == expected, read.prev)?;
    read.assert_end()
}

pub fn read_format(read: &mut CountingReader<impl Read>) -> Result<()> {
    let format = read.read_u32()?;
    assert_that!("format", format == FORMAT, read.prev)?;
    read.assert_end()
}

pub fn write_version(write: &mut CountingWriter<impl Write>, game: GameType) -> Result<()> {
    let version = match game {
        GameType::MW => VERSION_MW,
        GameType::PM => VERSION_PM,
        GameType::RC => return Err(assert_with_msg!("Recoil has no mechlib")),
        GameType::CS => return Err(assert_with_msg!("Crimson Skies has no mechlib")),
    };
    write.write_u32(version)?;
    Ok(())
}

pub fn write_format(write: &mut CountingWriter<impl Write>) -> Result<()> {
    write.write_u32(FORMAT)?;
    Ok(())
}

pub fn read_materials(read: &mut CountingReader<impl Read>) -> Result<Vec<Material>> {
    let count = read.read_u32()?;
    let materials = (0..count)
        .map(|index| {
            let material = read_material(read, index)?;
            Ok(match material {
                RawMaterial::Textured(mat) => {
                    // mechlib materials cannot have cycled textures
                    assert_that!("cycle ptr", mat.cycle_ptr == None, read.prev + 36)?;
                    // mechlib materials store the texture name immediately after
                    let texture = read.read_string()?;
                    Material::Textured(TexturedMaterial {
                        texture,
                        pointer: mat.pointer,
                        cycle: None,
                        specular: mat.specular,
                        flag: mat.flag,
                    })
                }
                RawMaterial::Colored(mat) => Material::Colored(mat),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    read.assert_end()?;
    Ok(materials)
}

pub fn write_materials(
    write: &mut CountingWriter<impl Write>,
    materials: &[Material],
) -> Result<()> {
    let materials_len = assert_len!(u32, materials.len(), "materials")?;
    write.write_u32(materials_len)?;

    for (index, material) in materials.iter().enumerate() {
        write_material(write, material, None, index)?;
        if let Material::Textured(textured) = material {
            if textured.cycle.is_some() {
                return Err(assert_with_msg!(
                    "mechlib materials cannot have cycled textures"
                ));
            }
            write.write_string(&textured.texture)?;
        }
    }
    Ok(())
}
