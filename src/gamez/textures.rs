use crate::assert::assert_utf8;
use crate::io_ext::{ReadHelper, WriteHelper};
use crate::size::ReprSize;
use crate::string::{str_from_c_suffix, str_to_c_suffix};
use crate::{assert_that, static_assert_size, Result};
use std::io::{Read, Write};

#[repr(C)]
struct TextureInfoC {
    zero00: u32,
    zero04: u32,
    texture: [u8; 20],
    used: u32,
    index: u32,
    unk36: i32,
}
static_assert_size!(TextureInfoC, 40);

pub fn read_texture_infos<R>(read: &mut R, offset: &mut u32, count: u32) -> Result<Vec<String>>
where
    R: Read,
{
    (0..count)
        .map(|_| {
            let info: TextureInfoC = read.read_struct()?;
            // not sure what this is. a pointer to the previous texture in the global
            // array? or a pointer to the texture?
            assert_that!("field 00", info.zero00 == 0, *offset + 0)?;
            // a non-zero value here causes additional dynamic code to be called
            assert_that!("field 04", info.zero04 == 0, *offset + 4)?;
            let texture = assert_utf8("texture", *offset + 8, || str_from_c_suffix(&info.texture))?;
            // 2 if the texture is used, 0 if the texture is unused
            // 1 or 3 if the texture is being processed (deallocated?)
            assert_that!("used", info.used == 2, *offset + 28)?;
            // stores the texture's index in the global texture array
            assert_that!("index", info.index == 0, *offset + 32)?;
            assert_that!("field 36", info.unk36 == -1, *offset + 36)?;
            *offset += TextureInfoC::SIZE;
            Ok(texture)
        })
        .collect::<Result<Vec<_>>>()
}

pub fn write_texture_infos<W>(write: &mut W, textures: &[String]) -> Result<()>
where
    W: Write,
{
    for name in textures {
        let mut texture = [0; 20];
        str_to_c_suffix(name, &mut texture);
        write.write_struct(&TextureInfoC {
            zero00: 0,
            zero04: 0,
            texture,
            used: 2,
            index: 0,
            unk36: -1,
        })?;
    }
    Ok(())
}

pub fn size_texture_infos(count: u32) -> u32 {
    TextureInfoC::SIZE * count
}
