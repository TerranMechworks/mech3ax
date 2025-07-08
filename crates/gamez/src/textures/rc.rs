use bytemuck::{AnyBitPattern, NoUninit};
use log::{trace, warn};
use mech3ax_api_types::gamez::Texture;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{chk, Result};
use mech3ax_types::check::suffix;
use mech3ax_types::{impl_as_bytes, primitive_enum, AsBytes as _, Ascii, Maybe, Offsets, Ptr};
use std::io::{Read, Write};

primitive_enum! {
    enum TextureState: u32 {
        Used = 2,
    }
}

type State = Maybe<u32, TextureState>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct TextureRcC {
    image_ptr: Ptr,   // 00
    surface_ptr: Ptr, // 04
    name: Ascii<20>,  // 08
    state: State,     // 28
    mip_index: i32,   // 32
}
impl_as_bytes!(TextureRcC, 36);

pub(crate) fn read_texture_directory(
    read: &mut CountingReader<impl Read>,
    count: i32,
) -> Result<Vec<Texture>> {
    (0..count)
        .map(|index| {
            trace!("Processing texture {}/{}", index, count);
            let texture: TextureRcC = read.read_struct()?;

            let offset = read.prev;

            chk!(offset, texture.image_ptr == Ptr::NULL)?;
            chk!(offset, texture.surface_ptr == Ptr::NULL)?;
            let name = chk!(offset, suffix(&texture.name))?;
            let _state = chk!(offset, ?texture.state)?;
            chk!(offset, texture.mip_index >= -1)?;
            chk!(offset, texture.mip_index < count)?;

            Ok(Texture {
                name,
                mip_index: texture.mip_index,
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
        trace!("Processing texture {}/{}", index, count);
        let name = Ascii::from_str_suffix(&texture.name);
        if texture.mip_index < -1 {
            warn!(
                "WARN: Expected texture mip index >= -1, but was {}",
                texture.mip_index
            );
        }

        let tex = TextureRcC {
            image_ptr: Ptr::NULL,
            surface_ptr: Ptr::NULL,
            name,
            state: TextureState::Used.maybe(),
            mip_index: texture.mip_index,
        };
        write.write_struct(&tex)?;
    }
    Ok(())
}

pub(crate) fn size_texture_directory(count: i32) -> u32 {
    TextureRcC::SIZE * (count as u32)
}
