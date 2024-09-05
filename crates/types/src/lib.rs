#![warn(clippy::all, clippy::cargo)]
mod as_bytes;
mod ascii;
pub mod bitflags;
mod bits;
mod byte_repr;
mod bytes;
mod cast;
mod debug_list;
mod enumerate;
mod hex;
pub mod primitive_enum;
mod ptr;
mod zeros;

pub use as_bytes::AsBytes;
pub use ascii::{str_from_ascii, str_to_ascii, string_from_ascii, Ascii};
pub use bitflags::{Bitflags, BitflagsRepr, Maybe};
// pub use bits::Bits;
pub use bytes::Bytes;
pub use cast::{u16_to_usize, u32_to_i64, u32_to_usize, AsUsize};
pub use enumerate::EnumerateEx;
pub use hex::Hex;
pub use primitive_enum::PrimitiveEnum;
pub use ptr::Ptr;
pub use zeros::Zeros;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionError {
    NonAscii(usize),
    PaddingError(&'static str),
    Unterminated,
}
