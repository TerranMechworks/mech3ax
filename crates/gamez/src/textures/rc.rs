use super::STATE_USED;
use bytemuck::{AnyBitPattern, NoUninit};
use log::{debug, trace};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::Ascii;
use mech3ax_types::{impl_as_bytes, AsBytes as _};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct TextureInfoRcC {
    zero00: u32,        // 00
    zero04: u32,        // 04
    texture: Ascii<20>, // 08
    used: u32,          // 28
    unk32: i32,         // 32
}
impl_as_bytes!(TextureInfoRcC, 36);

pub fn read_texture_infos(read: &mut CountingReader<impl Read>, count: u32) -> Result<Vec<String>> {
    (0..count)
        .map(|index| {
            debug!(
                "Reading texture info {} (rc, {}) at {}",
                index,
                TextureInfoRcC::SIZE,
                read.offset
            );
            let info: TextureInfoRcC = read.read_struct()?;
            trace!("{:#?}", info);

            assert_that!("field 00", info.zero00 == 0, read.prev + 0)?;
            assert_that!("field 04", info.zero04 == 0, read.prev + 4)?;
            let texture = assert_utf8("texture", read.prev + 8, || info.texture.to_str_suffix())?;
            assert_that!("field 28", info.used == STATE_USED, read.prev + 28)?;
            assert_that!("field 32", info.unk32 == -1, read.prev + 32)?;
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
            "Writing texture info {} (rc, {}) at {}",
            index,
            TextureInfoRcC::SIZE,
            write.offset
        );
        let texture = Ascii::from_str_suffix(name);
        let info = TextureInfoRcC {
            zero00: 0,
            zero04: 0,
            texture,
            used: STATE_USED,
            unk32: -1,
        };
        trace!("{:#?}", info);
        write.write_struct(&info)?;
    }
    Ok(())
}

pub fn size_texture_infos(count: u32) -> u32 {
    TextureInfoRcC::SIZE * count
}
