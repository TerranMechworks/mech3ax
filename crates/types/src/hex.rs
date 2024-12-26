#![allow(clippy::multiple_bound_locations)]
use bytemuck::{AnyBitPattern, NoUninit, TransparentWrapper};
use std::fmt;

pub trait HexDebug: NoUninit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl HexDebug for u8 {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // upper, prefixed hex
        write!(f, "0x{:04X}", self)
    }
}

impl HexDebug for u16 {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // upper, prefixed hex
        write!(f, "0x{:04X}", self)
    }
}

impl HexDebug for u32 {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // upper, prefixed hex
        write!(f, "0x{:08X}", self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, AnyBitPattern, TransparentWrapper, Default)]
#[repr(transparent)]
pub struct Hex<T: HexDebug>(pub T);

// SAFETY: `#[repr(transparent)]`, also only implemented for u8, u16, and u32.
unsafe impl<T: HexDebug> NoUninit for Hex<T> {}

impl<T: HexDebug> fmt::Debug for Hex<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        HexDebug::fmt(&self.0, f)
    }
}

impl<T: PartialEq + HexDebug> PartialEq<T> for Hex<T> {
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.0.eq(other)
    }
}
