use std::fmt;

pub trait BitsDebug {
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

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Bits<T: BitsDebug>(pub T);

impl<T: BitsDebug> fmt::Debug for Bits<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        BitsDebug::fmt(&self.0, f)
    }
}
