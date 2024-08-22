use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub(crate) struct HexByte(pub u8);

impl fmt::Debug for HexByte {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // lower, unprefixed hex
        write!(f, "{:02x}", self.0)
    }
}
