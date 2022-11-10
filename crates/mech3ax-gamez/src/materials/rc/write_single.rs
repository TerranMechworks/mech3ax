use super::{find_texture_index_by_name, MaterialC, MaterialFlags};
use log::{debug, trace};
use mech3ax_api_types::{Color, Material, ReprSize as _, TexturedMaterial};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{assert_with_msg, Result};
use std::io::Write;

const FLAGS_TEXTURES: u8 = MaterialFlags::TEXTURED.bits();
const FLAGS_COLORED: u8 = MaterialFlags::empty().bits();

fn validate_unused_fields(material: &TexturedMaterial) -> Result<()> {
    // unused fields in Recoil, avoid having another type
    if material.pointer != 0 {
        return Err(assert_with_msg!(
            "Expected `recoil material pointer` == 0, but was {}",
            material.pointer
        ));
    }
    if let Some(cycle) = &material.cycle {
        return Err(assert_with_msg!(
            "Expected `recoil material cycle` == None, but was {:#?}",
            cycle
        ));
    }
    if material.flag {
        return Err(assert_with_msg!(
            "Expected `recoil material flag` == false, but was true"
        ));
    }
    Ok(())
}

pub fn write_material(
    write: &mut CountingWriter<impl Write>,
    material: &Material,
    textures: &[String],
    index: i16,
) -> Result<()> {
    debug!(
        "Writing material {} ({}) at {}",
        index,
        MaterialC::SIZE,
        write.offset
    );
    let mat_c = match material {
        Material::Textured(material) => {
            validate_unused_fields(material)?;
            let index = find_texture_index_by_name(textures, &material.texture)?;
            MaterialC {
                alpha: 0xFF,
                flags: FLAGS_TEXTURES,
                rgb: 0x7FFF,
                color: Color::WHITE_FULL,
                index,
                zero20: 0.0,
                half24: 0.5,
                half28: 0.5,
                specular: material.specular,
                cycle_ptr: 0,
            }
        }
        Material::Colored(material) => MaterialC {
            alpha: material.alpha,
            flags: FLAGS_COLORED,
            rgb: 0x0000,
            color: material.color,
            index: 0,
            zero20: 0.0,
            half24: 0.5,
            half28: 0.5,
            specular: material.specular,
            cycle_ptr: 0,
        },
    };
    trace!("{:#?}", mat_c);
    write.write_struct(&mat_c)?;
    Ok(())
}
