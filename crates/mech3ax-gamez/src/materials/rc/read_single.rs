use super::{MaterialC, MaterialFlags};
use log::{debug, trace};
use mech3ax_api_types::{
    u32_to_usize, Color, ColoredMaterial, Material, ReprSize as _, TexturedMaterial,
};
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{assert_that, assert_with_msg, Result};
use std::io::Read;

pub fn read_material(
    read: &mut CountingReader<impl Read>,
    textures: &[String],
    index: i16,
) -> Result<Material> {
    debug!(
        "Reading material {} ({}) at {}",
        index,
        MaterialC::SIZE,
        read.offset
    );
    let material: MaterialC = read.read_struct()?;
    trace!("{:#?}", material);

    let bitflags = MaterialFlags::from_bits(material.flags).ok_or_else(|| {
        assert_with_msg!(
            "Expected valid material flags, but was 0x{:02X} (at {})",
            material.flags,
            read.prev + 1
        )
    })?;

    assert_that!("field 20", material.zero20 == 0.0, read.prev + 20)?;
    assert_that!("field 24", material.half24 == 0.5, read.prev + 24)?;
    assert_that!("field 28", material.half28 == 0.5, read.prev + 28)?;
    // Recoil doesn't have cycles
    assert_that!("cycle ptr", material.cycle_ptr == 0, read.prev + 36)?;

    let material = if bitflags.contains(MaterialFlags::TEXTURED) {
        // all color information comes from the texture
        assert_that!("alpha", material.alpha == 0xFF, read.prev + 0)?;
        assert_that!("rgb", material.rgb == 0x7FFF, read.prev + 2)?;
        assert_that!("color", material.color == Color::WHITE_FULL, read.prev + 4)?;

        let texture_index = u32_to_usize(material.index);
        assert_that!("texture index", texture_index < textures.len(), read.offset)?;
        let texture = textures[texture_index].clone();

        // because Recoil has no mechlib that stores the texture names after
        // the material, there is no need for a two-step read process.
        Material::Textured(TexturedMaterial {
            texture,
            specular: material.specular,
            // unused fields in Recoil, avoid having another type
            pointer: 0,
            cycle: None,
            flag: false,
        })
    } else {
        assert_that!("rgb", material.rgb == 0x0000, read.prev + 2)?;
        assert_that!("index", material.index == 0, read.prev + 16)?;

        Material::Colored(ColoredMaterial {
            color: material.color,
            alpha: material.alpha,
            specular: material.specular,
        })
    };
    Ok(material)
}

pub fn read_material_zero(read: &mut CountingReader<impl Read>, index: i16) -> Result<()> {
    debug!(
        "Reading zero material {} ({}) at {}",
        index,
        MaterialC::SIZE,
        read.offset
    );
    let material: MaterialC = read.read_struct()?;

    assert_that!("field 00", material.alpha == 0, read.prev + 0)?;
    assert_that!("flag", material.flags == 0, read.prev + 1)?;
    assert_that!("rgb", material.rgb == 0x0000, read.prev + 2)?;
    assert_that!("color", material.color == Color::BLACK, read.prev + 4)?;
    assert_that!("index", material.index == 0, read.prev + 16)?;
    assert_that!("field 20", material.zero20 == 0.0, read.prev + 20)?;
    assert_that!("field 24", material.half24 == 0.0, read.prev + 24)?;
    assert_that!("field 28", material.half28 == 0.0, read.prev + 28)?;
    assert_that!("specular", material.specular == 0.0, read.prev + 32)?;
    assert_that!("cycle ptr", material.cycle_ptr == 0, read.prev + 36)?;

    Ok(())
}
