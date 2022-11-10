use std::fmt;

pub trait HexDebug {
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

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Hex<T: HexDebug>(pub T);

impl<T: HexDebug> fmt::Debug for Hex<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        HexDebug::fmt(&self.0, f)
    }
}
