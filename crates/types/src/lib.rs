#![warn(clippy::all, clippy::cargo)]
mod as_bytes;
mod ascii;
mod bits;
mod byte_repr;
mod bytes;
mod cast;
mod debug_list;
mod hex;
mod ptr;
mod zeros;

pub use as_bytes::AsBytes;
pub use ascii::{str_from_ascii, str_to_ascii, string_from_ascii, Ascii};
pub use bits::Bits;
pub use bytes::Bytes;
pub use cast::{u16_to_usize, u32_to_usize, AsUsize};
pub use hex::{Hex, HexDebug};
pub use ptr::Ptr;
pub use zeros::Zeros;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionError {
    NonAscii(usize),
    PaddingError(&'static str),
    Unterminated,
}
