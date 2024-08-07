use bytemuck::{AnyBitPattern, NoUninit, TransparentWrapper};
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, NoUninit, AnyBitPattern, TransparentWrapper)]
#[repr(transparent)]
pub struct Ptr(pub u32);

impl Ptr {
    pub const NULL: Ptr = Ptr(0);
}

impl fmt::Debug for Ptr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // upper, prefixed hex
        write!(f, "0x{:08X}", self.0)
    }
}
