//! GameZ texture support for PM, CS
use super::STATE_USED;
use log::{debug, trace};
use mech3ax_api_types::{static_assert_size, ReprSize as _};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::string::{str_from_c_suffix, str_to_c_suffix};
use mech3ax_common::{assert_that, Result};
use mech3ax_debug::{Ascii, Ptr};
use std::io::{Read, Write};

#[derive(Debug)]
#[repr(C)]
struct TextureInfoNgC {
    unk00: Ptr,         // 00
    zero04: u32,        // 04
    zero08: u32,        // 08
    texture: Ascii<20>, // 12
    used: u32,          // 32
    index: u32,         // 36
    unk40: i32,         // 40
}
static_assert_size!(TextureInfoNgC, 44);

pub fn read_texture_infos(
    read: &mut CountingReader<impl Read>,
    count: u32,
) -> Result<(Vec<String>, Vec<Option<u32>>)> {
    let mut ptrs = Vec::with_capacity(count as _);
    let names = (0..count)
        .map(|index| {
            debug!(
                "Reading texture info {} (ng, {}) at {}",
                index,
                TextureInfoNgC::SIZE,
                read.offset
            );
            let info: TextureInfoNgC = read.read_struct()?;
            trace!("{:#?}", info);

            // validate field 00 later, with used
            assert_that!("field 04", info.zero04 == 0, read.prev + 4)?;
            assert_that!("field 08", info.zero08 == 0, read.prev + 8)?;
            let name = assert_utf8("texture", read.prev + 12, || {
                str_from_c_suffix(&info.texture.0)
            })?;
            // 2 if the texture is used, 0 if the texture is unused
            // 1 or 3 if the texture is being processed (deallocated?)
            assert_that!("field 32", info.used in [1, STATE_USED], read.prev + 32)?;
            let ptr = if info.used == STATE_USED {
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

pub fn write_texture_infos(
    write: &mut CountingWriter<impl Write>,
    textures: &[String],
    ptrs: &[Option<u32>],
) -> Result<()> {
    let ptrs = ptrs.iter().chain(std::iter::repeat(&None)).copied();
    for (index, (name, ptr)) in textures.iter().zip(ptrs).enumerate() {
        debug!(
            "Writing texture info {} (ng, {}) at {}",
            index,
            TextureInfoNgC::SIZE,
            write.offset
        );
        let mut texture = Ascii::new();
        str_to_c_suffix(name, &mut texture.0);
        let used = if ptr.is_some() { 1 } else { STATE_USED };
        let unk00 = Ptr(ptr.unwrap_or(0));
        let info = TextureInfoNgC {
            unk00,
            zero04: 0,
            zero08: 0,
            texture,
            used,
            index: 0,
            unk40: -1,
        };
        trace!("{:#?}", info);
        write.write_struct(&info)?;
    }
    Ok(())
}

pub fn size_texture_infos(count: u32) -> u32 {
    TextureInfoNgC::SIZE * count
}
