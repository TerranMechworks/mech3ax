use super::STATE_USED;
use bytemuck::{AnyBitPattern, NoUninit};
use log::trace;
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{impl_as_bytes, AsBytes as _, Ascii};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct TextureMwC {
    image_ptr: u32,   // 00
    surface_ptr: u32, // 04
    name: Ascii<20>,  // 08
    state: u32,       // 28
    zero32: u32,      // 32
    mip: i32,         // 36
}
impl_as_bytes!(TextureMwC, 40);

pub(crate) fn read_texture_directory(
    read: &mut CountingReader<impl Read>,
    count: i32,
) -> Result<Vec<String>> {
    (0..count)
        .map(|index| {
            trace!("Reading texture {}/{}", index, count);
            let info: TextureMwC = read.read_struct()?;

            assert_that!("image ptr", info.image_ptr == 0, read.prev + 0)?;
            assert_that!("surface ptr", info.surface_ptr == 0, read.prev + 4)?;
            let texture = assert_utf8("name", read.prev + 8, || info.name.to_str_suffix())?;
            // 2 if the texture is used, 0 if the texture is unused
            // 1 or 3 if the texture is being processed (deallocated?)
            assert_that!("state", info.state == STATE_USED, read.prev + 28)?;
            assert_that!("field 32", info.zero32 == 0, read.prev + 32)?;
            assert_that!("mip index", info.mip == -1, read.prev + 36)?;
            Ok(texture)
        })
        .collect::<Result<Vec<_>>>()
}

pub(crate) fn write_texture_directory(
    write: &mut CountingWriter<impl Write>,
    textures: &[String],
) -> Result<()> {
    let count = textures.len();
    for (index, texture) in textures.iter().enumerate() {
        trace!("Writing texture {}/{}", index, count);
        let name = Ascii::from_str_suffix(texture);
        let tex = TextureMwC {
            image_ptr: 0,
            surface_ptr: 0,
            name,
            state: STATE_USED,
            zero32: 0,
            mip: -1,
        };
        write.write_struct(&tex)?;
    }
    Ok(())
}

pub(crate) fn size_texture_directory(count: i32) -> u32 {
    TextureMwC::SIZE * (count as u32)
}
