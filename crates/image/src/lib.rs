#![warn(clippy::all, clippy::cargo)]
#![allow(clippy::identity_op)]
mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_types::{Ascii, Maybe, bitflags, impl_as_bytes};
pub use read::read_textures;
pub use write::write_textures;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct TexturesHeaderC {
    zero00: u32,               // 00
    has_entries: u32,          // 04
    global_palette_count: i32, // 08, this is an i32 to make the palette index calc easier
    texture_count: u32,        // 12
    zero16: u32,               // 16
    zero20: u32,               // 20
}
impl_as_bytes!(TexturesHeaderC, 24);

macro_rules! global_palette_len {
    () => {
        512
    };
}
pub(crate) use global_palette_len;
use mech3ax_api_types::image::TextureStretch;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct TextureEntryC {
    name: Ascii<32>,    // 00
    start_offset: u32,  // 32
    palette_index: i32, // 36
}
impl_as_bytes!(TextureEntryC, 40);

bitflags! {
    struct TexFlags: u32 {
        // if set, 2 bytes per pixel, else 1 byte per pixel
        const BYTES_PER_PIXEL2 = 1 << 0;    // 0x01
        const HAS_ALPHA = 1 << 1;           // 0x02
        const NO_ALPHA = 1 << 2;            // 0x04
        const FULL_ALPHA = 1 << 3;          // 0x08
        const GLOBAL_PALETTE = 1 << 4;      // 0x10
        // these are used internally to track allocated buffers
        // if these are set in the file, they can be ignored
        const IMAGE_LOADED = 1 << 5;        // 0x20
        const ALPHA_LOADED = 1 << 6;        // 0x40
        const PALETTE_LOADED = 1 << 7;      // 0x80
    }
}

type Flags = Maybe<u32, TexFlags>;
type Stretch = Maybe<u16, TextureStretch>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct TextureInfoC {
    flags: Flags,       // 00
    width: u16,         // 04
    height: u16,        // 06
    zero08: u32,        // 08
    palette_count: u16, // 12
    stretch: Stretch,   // 14
}
impl_as_bytes!(TextureInfoC, 16);
