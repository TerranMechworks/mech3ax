use crate::maybe::PrimitiveRepr;
use core::fmt;

impl crate::maybe::SupportsMaybe<u32> for i8 {
    #[inline]
    fn from_bits(v: u32) -> Option<Self> {
        if v & 0xFFFF_FF00 == 0 {
            Some(v as i8)
        } else {
            None
        }
    }

    fn fmt_value(v: u32, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if v & 0xFFFF_FF00 == 0 {
            let i = v as i8;
            write!(f, "{}", i)
        } else {
            write!(f, "0x{:08X}", v)
        }
    }

    #[inline]
    fn maybe(self) -> crate::maybe::Maybe<u32, Self> {
        crate::maybe::Maybe::new((self as u32) & 0x0000_00FF)
    }
}

impl crate::maybe::SupportsMaybe<u32> for u8 {
    #[inline]
    fn from_bits(v: u32) -> Option<Self> {
        if v & 0xFFFF_FF00 == 0 {
            Some(v as u8)
        } else {
            None
        }
    }

    fn fmt_value(v: u32, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if v & 0xFFFF_FF00 == 0 {
            let i = v as u8;
            write!(f, "{}", i)
        } else {
            write!(f, "0x{:08X}", v)
        }
    }

    #[inline]
    fn maybe(self) -> crate::maybe::Maybe<u32, Self> {
        crate::maybe::Maybe::new((self as u32) & 0x0000_00FF)
    }
}

pub trait Padded<R: PrimitiveRepr>: crate::maybe::SupportsMaybe<R> {
    const PATTERN: &'static str;
}

pub type PaddedI8 = crate::maybe::Maybe<u32, i8>;

impl Padded<u32> for i8 {
    const PATTERN: &'static str = "0x000000XX";
}

pub type PaddedU8 = crate::maybe::Maybe<u32, u8>;

impl Padded<u32> for u8 {
    const PATTERN: &'static str = "0x000000XX";
}

#[cfg(test)]
mod tests;
