#![allow(clippy::multiple_bound_locations)]
use bytemuck::{AnyBitPattern, NoUninit, TransparentWrapper};
use std::fmt;

pub trait BitsDebug: NoUninit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl BitsDebug for u8 {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0b{:08b}", self)
    }
}

impl BitsDebug for u16 {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0b{:016b}", self)
    }
}

impl BitsDebug for u32 {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0b{:032b}", self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, AnyBitPattern, TransparentWrapper)]
#[repr(transparent)]
pub struct Bits<T: BitsDebug>(pub T);

unsafe impl<T: BitsDebug> NoUninit for Bits<T> {}

impl<T: BitsDebug> fmt::Debug for Bits<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        BitsDebug::fmt(&self.0, f)
    }
}
