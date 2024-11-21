use super::STATE_USED;
use bytemuck::{AnyBitPattern, NoUninit};
use log::{debug, trace};
use mech3ax_api_types::{impl_as_bytes, AsBytes as _};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::string::{str_from_c_suffix, str_to_c_suffix};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::Ascii;
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct TextureInfoMwC {
    zero00: u32,        // 00
    zero04: u32,        // 04
    texture: Ascii<20>, // 08
    used: u32,          // 28
    index: u32,         // 32
    unk36: i32,         // 36
}
impl_as_bytes!(TextureInfoMwC, 40);

pub fn read_texture_infos(read: &mut CountingReader<impl Read>, count: u32) -> Result<Vec<String>> {
    (0..count)
        .map(|index| {
            debug!(
                "Reading texture info {} (mw, {}) at {}",
                index,
                TextureInfoMwC::SIZE,
                read.offset
            );
            let info: TextureInfoMwC = read.read_struct()?;
            trace!("{:#?}", info);

            assert_that!("field 00", info.zero00 == 0, read.prev + 0)?;
            assert_that!("field 04", info.zero04 == 0, read.prev + 4)?;
            let texture = assert_utf8("texture", read.prev + 8, || {
                str_from_c_suffix(&info.texture)
            })?;
            // 2 if the texture is used, 0 if the texture is unused
            // 1 or 3 if the texture is being processed (deallocated?)
            assert_that!("field 28", info.used == STATE_USED, read.prev + 28)?;
            // stores the texture's index in the global texture array
            assert_that!("field 32", info.index == 0, read.prev + 32)?;
            assert_that!("field 36", info.unk36 == -1, read.prev + 36)?;
            Ok(texture)
        })
        .collect::<Result<Vec<_>>>()
}

pub fn write_texture_infos(
    write: &mut CountingWriter<impl Write>,
    textures: &[String],
) -> Result<()> {
    for (index, name) in textures.iter().enumerate() {
        debug!(
            "Writing texture info {} (mw, {}) at {}",
            index,
            TextureInfoMwC::SIZE,
            write.offset
        );
        let mut texture = Ascii::zero();
        str_to_c_suffix(name, &mut texture);
        let info = TextureInfoMwC {
            zero00: 0,
            zero04: 0,
            texture,
            used: STATE_USED,
            index: 0,
            unk36: -1,
        };
        trace!("{:#?}", info);
        write.write_struct(&info)?;
    }
    Ok(())
}

pub fn size_texture_infos(count: u32) -> u32 {
    TextureInfoMwC::SIZE * count
}
