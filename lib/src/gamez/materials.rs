use crate::io_ext::{CountingReader, WriteHelper};
use crate::materials::{
    read_material, read_materials_zero, write_material, write_materials_zero, CycleData, Material,
    RawMaterial, TexturedMaterial, MATERIAL_C_SIZE,
};
use crate::size::ReprSize;
use crate::{assert_that, bool_c, static_assert_size, Result};
use std::io::{Read, Write};

#[repr(C)]
struct MaterialInfoC {
    array_size: i32,
    count: i32,
    index_max: i32,
    unknown: i32,
}
static_assert_size!(MaterialInfoC, 16);

#[repr(C)]
struct CycleInfoC {
    unk00: u32,
    unk04: u32,
    zero08: u32,
    unk12: f32,
    count1: u32,
    count2: u32,
    data_ptr: u32,
}
static_assert_size!(CycleInfoC, 28);

fn read_cycle<R>(
    read: &mut CountingReader<R>,
    material: RawMaterial,
    textures: &[String],
) -> Result<Material>
where
    R: Read,
{
    Ok(match material {
        RawMaterial::Textured(mat) => {
            let texture_index = mat.pointer as usize;
            assert_that!("texture index", texture_index < textures.len(), read.offset)?;
            let texture = textures[texture_index].clone();

            let cycle = if let Some(info_ptr) = mat.cycle_ptr {
                assert_that!("cycle info ptr", info_ptr != 0, read.prev + 0)?;

                let info: CycleInfoC = read.read_struct()?;
                let unk00 = assert_that!("field 00", bool info.unk00, read.prev + 0)?;
                assert_that!("field 08", info.zero08 == 0, read.prev + 8)?;
                assert_that!("field 12", 2.0 <= info.unk12 <= 16.0, read.prev + 12)?;
                assert_that!("cycle count", info.count1 == info.count2, read.prev + 20)?;
                assert_that!("cycle data ptr", info.data_ptr != 0, read.prev + 24)?;

                let textures = (0..info.count1)
                    .map(|_| {
                        let texture_index = read.read_u32()? as usize;
                        assert_that!("texture index", texture_index < textures.len(), read.prev)?;
                        let texture = textures[texture_index].clone();
                        Ok(texture)
                    })
                    .collect::<Result<Vec<_>>>()?;

                Some(CycleData {
                    textures,
                    info_ptr,
                    data_ptr: info.data_ptr,
                    unk00,
                    unk04: info.unk04,
                    unk12: info.unk12,
                })
            } else {
                None
            };

            Material::Textured(TexturedMaterial {
                texture,
                // since this stores the index of the texture name, zero it out
                pointer: 0,
                cycle,
                unk32: mat.unk32,
                flag: mat.flag,
            })
        }
        RawMaterial::Colored(mat) => Material::Colored(mat),
    })
}

pub fn read_materials<R>(
    read: &mut CountingReader<R>,
    textures: &[String],
) -> Result<(Vec<Material>, i16)>
where
    R: Read,
{
    let info: MaterialInfoC = read.read_struct()?;
    assert_that!("mat array size", 0 <= info.array_size <= i16::MAX as i32, read.prev + 0)?;
    assert_that!("mat count", 0 <= info.count <= info.array_size, read.prev + 0)?;
    assert_that!("mat index max", info.index_max == info.count, read.prev + 8)?;
    assert_that!(
        "mat field 12",
        info.unknown == info.count - 1,
        read.prev + 12
    )?;

    let count = info.count as i16;
    let array_size = info.array_size as i16;

    // read materials without cycle data
    let materials = (0..count)
        .map(|index| {
            let material = read_material(read)?;

            let mut expected_index1 = index + 1;
            if expected_index1 >= count {
                expected_index1 = -1;
            }
            let actual_index1 = read.read_i16()?;
            assert_that!("mat index 1", actual_index1 == expected_index1, read.prev)?;

            let mut expected_index2 = index - 1;
            if expected_index2 < 0 {
                expected_index2 = -1;
            }
            let actual_index2 = read.read_i16()?;
            assert_that!("mat index 2", actual_index2 == expected_index2, read.prev)?;

            Ok(material)
        })
        .collect::<Result<Vec<_>>>()?;

    read_materials_zero(read, count, array_size)?;

    // now read cycle data
    let materials = materials
        .into_iter()
        .map(|material| read_cycle(read, material, textures))
        .collect::<Result<Vec<_>>>()?;

    Ok((materials, array_size))
}

fn write_cycle<W>(write: &mut W, textures: &[String], material: &Material) -> Result<()>
where
    W: Write,
{
    if let Material::Textured(mat) = material {
        if let Some(cycle) = &mat.cycle {
            let unk00 = bool_c!(cycle.unk00);
            let count = cycle.textures.len() as u32;
            write.write_struct(&CycleInfoC {
                unk00,
                unk04: cycle.unk04,
                zero08: 0,
                unk12: cycle.unk12,
                count1: count,
                count2: count,
                data_ptr: cycle.data_ptr,
            })?;

            for texture in &cycle.textures {
                let texture_index = textures
                    .iter()
                    .position(|name| name == texture)
                    .expect("Texture name not found") as u32;
                write.write_u32(texture_index)?;
            }
        }
    }
    Ok(())
}

pub fn write_materials<W>(
    write: &mut W,
    textures: &[String],
    materials: &[Material],
    array_size: i16,
) -> Result<()>
where
    W: Write,
{
    let count = materials.len() as i32;
    write.write_struct(&MaterialInfoC {
        array_size: array_size as i32,
        count,
        index_max: count,
        unknown: count - 1,
    })?;

    let count = materials.len() as i16;
    for (i, material) in materials.iter().enumerate() {
        let pointer = if let Material::Textured(textured) = material {
            // reconstruct the texture index
            let texture_index = textures
                .iter()
                .position(|tex| tex == &textured.texture)
                .expect("Texture name not found");
            Some(texture_index as u32)
        } else {
            None
        };
        write_material(write, material, pointer)?;

        let index = i as i16;
        let mut index1 = index + 1;
        if index1 >= count {
            index1 = -1;
        }
        write.write_i16(index1)?;

        let mut index2 = index - 1;
        if index2 < 0 {
            index2 = -1;
        }
        write.write_i16(index2)?;
    }

    write_materials_zero(write, count, array_size)?;

    for material in materials {
        write_cycle(write, textures, material)?;
    }
    Ok(())
}

pub fn size_materials(array_size: i16, materials: &[Material]) -> u32 {
    let mut size = MaterialInfoC::SIZE + (MATERIAL_C_SIZE + 2 + 2) * array_size as u32;
    for material in materials {
        if let Material::Textured(mat) = material {
            if let Some(cycle) = &mat.cycle {
                size += CycleInfoC::SIZE + (cycle.textures.len() as u32) * 4;
            }
        }
    }
    size
}
