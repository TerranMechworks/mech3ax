//! GameZ texture support for PM, CS
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
        Assigned = 1,
        Used = 2,
    }
}

type State = Maybe<u32, TextureState>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct TexturePmC {
    image_ptr: Ptr,   // 00
    field04: u32,     // 04
    surface_ptr: Ptr, // 08
    name: Ascii<20>,  // 12
    state: State,     // 32
    category: i32,    // 36
    mip_index: i32,   // 40
}
impl_as_bytes!(TexturePmC, 44);

pub(crate) fn read_texture_directory(
    read: &mut CountingReader<impl Read>,
    count: i32,
) -> Result<Vec<Texture>> {
    (0..count)
        .map(|index| {
            trace!("Processing texture {}/{}", index, count);
            let texture: TexturePmC = read.read_struct()?;

            let offset = read.prev;

            chk!(offset, texture.field04 == 0)?;
            chk!(offset, texture.surface_ptr == Ptr::NULL)?;
            let name = chk!(offset, suffix(&texture.name))?;
            let state = chk!(offset, ?texture.state)?;
            match state {
                TextureState::Assigned => {
                    chk!(offset, texture.image_ptr != Ptr::NULL)?;
                }
                TextureState::Used => {
                    // somehow, this is now the rarer case
                    chk!(offset, texture.image_ptr == Ptr::NULL)?;
                }
            }
            chk!(offset, texture.category == 0)?;
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
    image_ptrs: &[u32],
) -> Result<()> {
    let ptrs = image_ptrs
        .iter()
        .copied()
        .map(Ptr)
        .chain(std::iter::repeat(Ptr::NON_NULL));

    let count = textures.len();
    for (index, (texture, image_ptr)) in textures.iter().zip(ptrs).enumerate() {
        trace!("Processing texture {}/{}", index, count);
        let name = Ascii::from_str_suffix(&texture.name);
        if texture.mip_index < -1 {
            warn!(
                "WARN: Expected texture mip index >= -1, but was {}",
                texture.mip_index
            );
        }

        let state = if image_ptr == Ptr::NULL {
            TextureState::Used
        } else {
            TextureState::Assigned
        };

        let tex = TexturePmC {
            image_ptr,
            field04: 0,
            surface_ptr: Ptr::NULL,
            name,
            state: state.maybe(),
            category: 0,
            mip_index: texture.mip_index,
        };
        write.write_struct(&tex)?;
    }
    Ok(())
}

pub(crate) fn size_texture_directory(count: i32) -> u32 {
    TexturePmC::SIZE * (count as u32)
}
