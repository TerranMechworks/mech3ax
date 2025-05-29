use super::STATE_USED;
use bytemuck::{AnyBitPattern, NoUninit};
use log::{trace, warn};
use mech3ax_api_types::gamez::Texture;
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{impl_as_bytes, AsBytes as _, Ascii};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct TextureInfoRcC {
    image_ptr: u32,  // 00
    zero04: u32,     // 04
    name: Ascii<20>, // 08
    state: u32,      // 28
    mip: i32,        // 32
}
impl_as_bytes!(TextureInfoRcC, 36);

pub(crate) fn read_texture_directory(
    read: &mut CountingReader<impl Read>,
    count: i32,
) -> Result<Vec<Texture>> {
    (0..count)
        .map(|index| {
            trace!("Reading texture info {}/{}", index, count);
            let info: TextureInfoRcC = read.read_struct()?;

            assert_that!("image ptr", info.image_ptr == 0, read.prev + 0)?;
            assert_that!("field 04", info.zero04 == 0, read.prev + 4)?;
            let name = assert_utf8("texture", read.prev + 8, || info.name.to_str_suffix())?;
            assert_that!("state", info.state == STATE_USED, read.prev + 28)?;

            assert_that!("mip index", info.mip >= -1, read.prev + 32)?;

            Ok(Texture {
                name,
                mip: info.mip,
            })
        })
        .collect::<Result<Vec<_>>>()
}

pub(crate) fn write_texture_directory(
    write: &mut CountingWriter<impl Write>,
    textures: &[Texture],
) -> Result<()> {
    let count = textures.len();
    for (index, texture) in textures.iter().enumerate() {
        trace!("Writing texture info {}/{}", index, count);
        let name = Ascii::from_str_suffix(&texture.name);
        if texture.mip < -1 {
            warn!(
                "WARN: Expected texture mip index >= -1, but was {}",
                texture.mip
            );
        }
        let info = TextureInfoRcC {
            image_ptr: 0,
            zero04: 0,
            name,
            state: STATE_USED,
            mip: texture.mip,
        };
        write.write_struct(&info)?;
    }
    Ok(())
}

pub(crate) fn size_texture_infos(count: i32) -> u32 {
    TextureInfoRcC::SIZE * (count as u32)
}
