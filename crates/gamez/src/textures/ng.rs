//! GameZ texture support for PM, CS
use super::{STATE_UNUSED, STATE_USED};
use bytemuck::{AnyBitPattern, NoUninit};
use log::trace;
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{impl_as_bytes, u32_to_usize, AsBytes as _, Ascii, Ptr};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct TextureInfoNgC {
    unk00: Ptr,         // 00
    zero04: u32,        // 04
    zero08: u32,        // 08
    texture: Ascii<20>, // 12
    state: u32,         // 32
    index: u32,         // 36
    unk40: i32,         // 40
}
impl_as_bytes!(TextureInfoNgC, 44);

pub(crate) fn read_texture_infos(
    read: &mut CountingReader<impl Read>,
    count: u32,
) -> Result<(Vec<String>, Vec<Option<u32>>)> {
    let mut ptrs = Vec::with_capacity(u32_to_usize(count));
    let names = (0..count)
        .map(|index| {
            trace!("Reading texture info {}/{}", index, count);
            let info: TextureInfoNgC = read.read_struct()?;

            // validate field 00 later, with used
            assert_that!("field 04", info.zero04 == 0, read.prev + 4)?;
            assert_that!("field 08", info.zero08 == 0, read.prev + 8)?;
            let name = assert_utf8("texture", read.prev + 12, || info.texture.to_str_suffix())?;
            // 2 if the texture is used, 0 if the texture is unused
            // 1 or 3 if the texture is being processed (deallocated?)
            assert_that!("field 32", info.state in [STATE_UNUSED, STATE_USED], read.prev + 32)?;
            let ptr = if info.state == STATE_USED {
                // somehow, this is now the rarer case
                assert_that!("field 00", info.unk00 == Ptr::NULL, read.prev + 0)?;
                None
            } else {
                // not sure what this is. a pointer to the previous texture in the global
                // array? or a pointer to the texture?
                assert_that!("field 00", info.unk00 != Ptr::NULL, read.prev + 0)?;
                Some(info.unk00.0)
            };

            assert_that!("field 36", info.index == 0, read.prev + 36)?;
            assert_that!("field 40", info.unk40 == -1, read.prev + 40)?;

            ptrs.push(ptr);
            Ok(name)
        })
        .collect::<Result<Vec<_>>>()?;
    Ok((names, ptrs))
}

pub(crate) fn write_texture_infos(
    write: &mut CountingWriter<impl Write>,
    textures: &[String],
    ptrs: &[Option<u32>],
) -> Result<()> {
    let ptrs = ptrs
        .iter()
        .chain(std::iter::repeat(&Some(u32::MAX)))
        .copied();
    let count = textures.len();
    for (index, (name, ptr)) in textures.iter().zip(ptrs).enumerate() {
        trace!("Writing texture info {}/{}", index, count);
        let texture = Ascii::from_str_suffix(name);
        let state = if ptr.is_some() {
            STATE_UNUSED
        } else {
            STATE_USED
        };
        let unk00 = Ptr(ptr.unwrap_or(0));
        let info = TextureInfoNgC {
            unk00,
            zero04: 0,
            zero08: 0,
            texture,
            state,
            index: 0,
            unk40: -1,
        };
        write.write_struct(&info)?;
    }
    Ok(())
}

pub(crate) fn size_texture_infos(count: u32) -> u32 {
    TextureInfoNgC::SIZE * count
}
