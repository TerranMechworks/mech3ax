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
mod ptr;
mod zeros;

pub use as_bytes::AsBytes;
pub use ascii::{str_from_ascii, str_to_ascii, string_from_ascii, Ascii};
pub use bitflags::{Bitflags, BitflagsRepr, Maybe};
// pub use bits::Bits;
pub use bytes::Bytes;
pub use cast::{u16_to_usize, u32_to_usize, AsUsize};
pub use enumerate::EnumerateEx;
pub use hex::Hex;
pub use ptr::Ptr;
pub use zeros::Zeros;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionError {
    NonAscii(usize),
    PaddingError(&'static str),
    Unterminated,
}

/// A trait for enums that can be converted from/to a primitive type (e.g. u32).
/// This trait should not be implemented manually; instead it is intended to be
/// derived by the corresponding proc-macro!
///
/// Three methods are derived:
/// * `fn PrimitiveEnum::from_primitive(v: <primitive>) -> Option<Self>` for
///   trying to convert a primitive type to the enum.
/// * `From<Self> for <primitive>` for converting the enum to a primitive type
///   (`into()`).
/// * `const fn as_(self) -> <primitive>` for converting the enum to a
///   primitive type (`const`).
///
/// The corresponding proc-macro also gathers all valid discriminants for nice
/// assertions.
pub trait PrimitiveEnum: Sized + Sync + Send + 'static + Into<Self::Primitive> {
    type Primitive: Copy + std::fmt::Display;
    const DISCRIMINANTS: &'static str;

    fn from_primitive(v: Self::Primitive) -> Option<Self>;
}

pub use mech3ax_types_proc_macro::PrimitiveEnum;
