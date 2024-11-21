use crate::byte_repr::HexByte;
use crate::debug_list::DebugList;
use bytemuck::{AnyBitPattern, NoUninit, TransparentWrapper, Zeroable};
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Bytes<const N: usize>(pub [u8; N]);

// SAFETY: `#[repr(transparent)]`.
unsafe impl<const N: usize> TransparentWrapper<[u8; N]> for Bytes<N> {}

// SAFETY: An array of u8 is obviously zero-able.
unsafe impl<const N: usize> Zeroable for Bytes<N> {
    #[inline]
    fn zeroed() -> Self {
        Self::new()
    }
}

// SAFETY: An array of u8 is valid for any combination of bits.
unsafe impl<const N: usize> AnyBitPattern for Bytes<N> {}

// SAFETY: This is complicated
// * For any T that is Pod, an array of T is also Pod:
//   `unsafe impl<T, const N: usize> Pod for [T; N] where T: Pod {}`
// * For any T that is Pod, T is also NoUninit:
//   `unsafe impl<T: Pod> NoUninit for T {}`
// * Therefore:
//   `unsafe impl<T, const N: usize> NoUninit for [T; N] where T: NoUninit {}`
// * `u8` is obviously Pod/NoUninit:
//   `unsafe impl Pod for u8 {}`
// * Finally, the type is `#[repr(transparent)]`
unsafe impl<const N: usize> NoUninit for Bytes<N> {}

impl<const N: usize> Bytes<N> {
    #[inline]
    pub const fn new() -> Self {
        Self([0u8; N])
    }

    #[inline]
    pub fn to_vec(&self) -> Vec<u8> {
        Vec::from(self.0)
    }

    #[inline]
    pub fn from_slice(slice: &[u8]) -> Self {
        let mut s = Self::new();
        let len = slice.len();
        if len > N {
            s.0.copy_from_slice(&slice[..N]);
        } else {
            (&mut s.0[..len]).copy_from_slice(&slice);
        }
        s
    }
}

impl<const N: usize> Default for Bytes<N> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> fmt::Debug for Bytes<N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.0.iter().copied().map(HexByte);
        DebugList::new(f).entries(entries).finish()
    }
}

impl<const N: usize> From<[u8; N]> for Bytes<N> {
    #[inline]
    fn from(inner: [u8; N]) -> Self {
        Self(inner)
    }
}

impl<const N: usize> AsRef<[u8]> for Bytes<N> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<const N: usize> AsRef<[u8; N]> for Bytes<N> {
    #[inline]
    fn as_ref(&self) -> &[u8; N] {
        &self.0
    }
}

impl<const N: usize> std::ops::Deref for Bytes<N> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> PartialEq<&Bytes<N>> for Bytes<N> {
    #[inline]
    fn eq(&self, other: &&Bytes<N>) -> bool {
        self.eq(*other)
    }
}

impl<const N: usize> PartialEq<[u8]> for Bytes<N> {
    #[inline]
    fn eq(&self, other: &[u8]) -> bool {
        self.0.eq(other)
    }
}

impl<const N: usize> PartialEq<&[u8]> for Bytes<N> {
    #[inline]
    fn eq(&self, other: &&[u8]) -> bool {
        self.0.eq(*other)
    }
}

impl<const N: usize> PartialEq<[u8; N]> for Bytes<N> {
    #[inline]
    fn eq(&self, other: &[u8; N]) -> bool {
        self.0.eq(other)
    }
}

impl<const N: usize> PartialEq<&[u8; N]> for Bytes<N> {
    #[inline]
    fn eq(&self, other: &&[u8; N]) -> bool {
        self.0.eq(*other)
    }
}

#[cfg(test)]
mod tests;
