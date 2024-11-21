use bytemuck::{AnyBitPattern, NoUninit, TransparentWrapper};
use std::fmt;

/// A type for 32-bit pointer values.
///
/// The representation is always hexadecimal.
#[derive(Clone, Copy, PartialEq, Eq, NoUninit, AnyBitPattern, TransparentWrapper)]
#[repr(transparent)]
pub struct Ptr(pub u32);

impl Ptr {
    pub const NULL: Ptr = Ptr(0);
    pub const INVALID: Ptr = Ptr(u32::MAX);
}

impl fmt::Debug for Ptr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // upper, prefixed hex
        write!(f, "0x{:08X}", self.0)
    }
}

impl Default for Ptr {
    #[inline]
    fn default() -> Self {
        Self::NULL
    }
}

impl From<u32> for Ptr {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<Ptr> for u32 {
    #[inline]
    fn from(value: Ptr) -> Self {
        value.0
    }
}

impl AsRef<u32> for Ptr {
    #[inline]
    fn as_ref(&self) -> &u32 {
        &self.0
    }
}

impl PartialEq<&Ptr> for Ptr {
    #[inline]
    fn eq(&self, other: &&Ptr) -> bool {
        self.eq(*other)
    }
}

impl PartialEq<u32> for Ptr {
    #[inline]
    fn eq(&self, other: &u32) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<&u32> for Ptr {
    #[inline]
    fn eq(&self, other: &&u32) -> bool {
        self.0.eq(*other)
    }
}
