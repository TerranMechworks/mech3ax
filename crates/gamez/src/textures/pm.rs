//! GameZ texture support for PM, CS
use bytemuck::{AnyBitPattern, NoUninit};
use log::{trace, warn};
use mech3ax_api_types::gamez::Texture;
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{impl_as_bytes, primitive_enum, AsBytes as _, Ascii, Maybe, Ptr};
use std::io::{Read, Write};

primitive_enum! {
    enum TextureState: u32 {
        Assigned = 1,
        Used = 2,
    }
}

type State = Maybe<u32, TextureState>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct TexturePmC {
    image_ptr: Ptr,   // 00
    zero04: u32,      // 04
    surface_ptr: Ptr, // 08
    name: Ascii<20>,  // 12
    state: State,     // 32
    category: u32,    // 36
    mip: i32,         // 40
}
impl_as_bytes!(TexturePmC, 44);

pub(crate) fn read_texture_directory(
    read: &mut CountingReader<impl Read>,
    count: i32,
) -> Result<Vec<Texture>> {
    (0..count)
        .map(|index| {
            trace!("Processing texture {}/{}", index, count);
            let tex: TexturePmC = read.read_struct()?;

            assert_that!("field 04", tex.zero04 == 0, read.prev + 4)?;
            assert_that!("surface ptr", tex.surface_ptr == Ptr::NULL, read.prev + 8)?;
            let name = assert_utf8("name", read.prev + 12, || tex.name.to_str_suffix())?;
            let state = assert_that!("state", enum tex.state, read.prev + 32)?;
            match state {
                TextureState::Assigned => {
                    assert_that!("image ptr", tex.image_ptr != Ptr::NULL, read.prev + 0)?;
                }
                TextureState::Used => {
                    // somehow, this is now the rarer case
                    assert_that!("image ptr", tex.image_ptr == Ptr::NULL, read.prev + 0)?;
                }
            }
            assert_that!("category", tex.category == 0, read.prev + 36)?;
            assert_that!("mip index", tex.mip >= -1, read.prev + 40)?;

            Ok(Texture { name, mip: tex.mip })
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
        if texture.mip < -1 {
            warn!(
                "WARN: Expected texture mip index >= -1, but was {}",
                texture.mip
            );
        }

        let state = if image_ptr == Ptr::NULL {
            TextureState::Used
        } else {
            TextureState::Assigned
        };

        let tex = TexturePmC {
            image_ptr,
            zero04: 0,
            surface_ptr: Ptr::NULL,
            name,
            state: state.maybe(),
            category: 0,
            mip: texture.mip,
        };
        write.write_struct(&tex)?;
    }
    Ok(())
}

pub(crate) fn size_texture_directory(count: i32) -> u32 {
    TexturePmC::SIZE * (count as u32)
}
