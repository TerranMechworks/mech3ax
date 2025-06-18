use super::{State, TextureState};
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
struct TextureRcC {
    image_ptr: u32,   // 00
    surface_ptr: u32, // 04
    name: Ascii<20>,  // 08
    state: State,     // 28
    mip: i32,         // 32
}
impl_as_bytes!(TextureRcC, 36);

pub(crate) fn read_texture_directory(
    read: &mut CountingReader<impl Read>,
    count: i32,
) -> Result<Vec<Texture>> {
    (0..count)
        .map(|index| {
            trace!("Reading texture {}/{}", index, count);
            let tex: TextureRcC = read.read_struct()?;

            assert_that!("image ptr", tex.image_ptr == 0, read.prev + 0)?;
            assert_that!("surface ptr", tex.surface_ptr == 0, read.prev + 4)?;
            let name = assert_utf8("name", read.prev + 8, || tex.name.to_str_suffix())?;
            assert_that!("state", enum tex.state, read.prev + 28)?;
            assert_that!("mip index", tex.mip >= -1, read.prev + 32)?;

            Ok(Texture { name, mip: tex.mip })
        })
        .collect::<Result<Vec<_>>>()
}

pub(crate) fn write_texture_directory(
    write: &mut CountingWriter<impl Write>,
    textures: &[Texture],
) -> Result<()> {
    let count = textures.len();
    for (index, texture) in textures.iter().enumerate() {
        trace!("Writing texture {}/{}", index, count);
        let name = Ascii::from_str_suffix(&texture.name);
        if texture.mip < -1 {
            warn!(
                "WARN: Expected texture mip index >= -1, but was {}",
                texture.mip
            );
        }

        let tex = TextureRcC {
            image_ptr: 0,
            surface_ptr: 0,
            name,
            state: TextureState::Used.maybe(),
            mip: texture.mip,
        };
        write.write_struct(&tex)?;
    }
    Ok(())
}

pub(crate) fn size_texture_directory(count: i32) -> u32 {
    TextureRcC::SIZE * (count as u32)
}
