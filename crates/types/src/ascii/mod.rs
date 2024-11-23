mod conv;

use crate::byte_repr::AsciiByte;
use crate::debug_list::DebugList;
use bytemuck::{AnyBitPattern, NoUninit, TransparentWrapper, Zeroable};
pub use conv::{str_from_ascii, str_to_ascii, string_from_ascii};
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Ascii<const N: usize>([u8; N]);

// SAFETY: `#[repr(transparent)]`.
unsafe impl<const N: usize> TransparentWrapper<[u8; N]> for Ascii<N> {}

// SAFETY: An array of u8 is obviously zero-able.
unsafe impl<const N: usize> Zeroable for Ascii<N> {
    #[inline]
    fn zeroed() -> Self {
        Self([0u8; N])
    }
}

// SAFETY: An array of u8 is valid for any combination of bits.
unsafe impl<const N: usize> AnyBitPattern for Ascii<N> {}

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
unsafe impl<const N: usize> NoUninit for Ascii<N> {}

impl<const N: usize> Ascii<N> {
    #[inline]
    pub const fn zero() -> Self {
        Self([0u8; N])
    }

    #[inline]
    pub const fn new(inner: &[u8; N]) -> Self {
        Self(*inner)
    }

    #[inline]
    pub fn copy_from(&mut self, from: &[u8; N]) {
        self.0.copy_from_slice(from);
    }

    #[inline]
    pub fn first_is_zero(&self) -> bool {
        self.0[0] == 0
    }

    #[inline]
    pub fn escape_ascii(&self) -> core::slice::EscapeAscii<'_> {
        self.0.escape_ascii()
    }
}

impl<const N: usize> Default for Ascii<N> {
    #[inline]
    fn default() -> Self {
        Self::zero()
    }
}

impl<const N: usize> fmt::Debug for Ascii<N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.0.iter().copied().map(AsciiByte);
        DebugList::new(f).entries(entries).finish()
    }
}

impl<const N: usize> From<[u8; N]> for Ascii<N> {
    #[inline]
    fn from(inner: [u8; N]) -> Self {
        Self(inner)
    }
}

impl<const N: usize> AsRef<[u8]> for Ascii<N> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<const N: usize> AsRef<[u8; N]> for Ascii<N> {
    #[inline]
    fn as_ref(&self) -> &[u8; N] {
        &self.0
    }
}

impl<const N: usize> std::ops::Deref for Ascii<N> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> PartialEq<&Ascii<N>> for Ascii<N> {
    #[inline]
    fn eq(&self, other: &&Ascii<N>) -> bool {
        self.eq(*other)
    }
}

impl<const N: usize> PartialEq<[u8]> for Ascii<N> {
    #[inline]
    fn eq(&self, other: &[u8]) -> bool {
        self.0.eq(other)
    }
}

impl<const N: usize> PartialEq<&[u8]> for Ascii<N> {
    #[inline]
    fn eq(&self, other: &&[u8]) -> bool {
        self.0.eq(*other)
    }
}

impl<const N: usize> PartialEq<[u8; N]> for Ascii<N> {
    #[inline]
    fn eq(&self, other: &[u8; N]) -> bool {
        self.0.eq(other)
    }
}

impl<const N: usize> PartialEq<&[u8; N]> for Ascii<N> {
    #[inline]
    fn eq(&self, other: &&[u8; N]) -> bool {
        self.0.eq(*other)
    }
}

impl Ascii<64> {
    pub fn split(&self) -> (Ascii<32>, Ascii<32>) {
        let mut l = Ascii::zero();
        let mut r = Ascii::zero();
        l.0.copy_from_slice(&self.0[..32]);
        r.0.copy_from_slice(&self.0[32..]);
        (l, r)
    }

    pub fn combine(l: &Ascii<32>, r: &Ascii<32>) -> Self {
        let mut s = Ascii::zero();
        let (left, right) = s.0.split_at_mut(32);
        left.copy_from_slice(&l.0);
        right.copy_from_slice(&r.0);
        s
    }
}

#[cfg(test)]
mod tests;
