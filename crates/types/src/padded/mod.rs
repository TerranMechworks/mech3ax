use crate::maybe::{Maybe, SupportsMaybe};
use std::fmt;

pub type PaddedI8 = Maybe<u32, i8>;
pub type PaddedU8 = Maybe<u32, u8>;

impl SupportsMaybe<u32> for i8 {
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
    fn maybe(self) -> Maybe<u32, Self> {
        Maybe::new((self as u32) & 0x0000_00FF)
    }

    #[inline]
    fn check(v: u32) -> Result<Self, String> {
        Self::from_bits(v).ok_or_else(|| format!("expected {} to have padding 0x000000XX", v))
    }
}

impl SupportsMaybe<u32> for u8 {
    #[inline]
    fn from_bits(v: u32) -> Option<Self> {
        if v & 0xFFFF_FF00 == 0 {
            Some(v as u8)
        } else {
            None
        }
    }

    #[inline]
    fn fmt_value(v: u32, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if v & 0xFFFF_FF00 == 0 {
            let i = v as u8;
            write!(f, "{}", i)
        } else {
            write!(f, "0x{:08X}", v)
        }
    }

    #[inline]
    fn maybe(self) -> Maybe<u32, Self> {
        Maybe::new((self as u32) & 0x0000_00FF)
    }

    #[inline]
    fn check(v: u32) -> Result<Self, String> {
        Self::from_bits(v).ok_or_else(|| format!("expected {} to have padding 0x000000XX", v))
    }
}

#[cfg(test)]
mod tests;
