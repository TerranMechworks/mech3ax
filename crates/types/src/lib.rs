#![warn(clippy::all, clippy::cargo)]
mod as_bytes;
mod ascii;
pub mod bitflags;
pub mod boolean;
mod byte_repr;
mod bytes;
mod cast;
pub mod check;
pub mod cstruct;
mod debug_list;
mod enumerate;
mod hex;
pub mod maybe;
mod padded;
pub mod primitive_enum;
mod ptr;
mod zeros;

pub use as_bytes::AsBytes;
pub use ascii::{Ascii, str_from_ascii, str_to_ascii, string_from_ascii};
pub use boolean::{Bool8, Bool16, Bool32};
pub use bytes::Bytes;
pub use cast::{AsUsize, i32_to_usize, u16_to_usize, u32_to_i64, u32_to_usize};
pub use enumerate::EnumerateEx;
pub use hex::Hex;
pub use maybe::{Maybe, SupportsMaybe};
pub use mech3ax_types_proc_macro::{Offsets, json_flags};
pub use padded::{PaddedI8, PaddedU8};
pub use ptr::Ptr;
pub use zeros::Zeros;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionError {
    NonAscii(usize),
    PaddingError(&'static str),
    Unterminated,
}
