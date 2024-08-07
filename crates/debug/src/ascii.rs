use crate::byte_repr::AsciiByte;
use crate::debug_list::DebugList;
use bytemuck::{AnyBitPattern, NoUninit, TransparentWrapper, Zeroable};
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Ascii<const N: usize>(pub [u8; N]);

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

#[cfg(test)]
mod tests {
    use super::Ascii;

    #[test]
    fn ascii_all_ascii() {
        let a = Ascii::new(b"hello!");
        let f = format!("{:?}", a);
        assert_eq!(f, "[h, e, l, l, o, !]");
    }

    #[test]
    fn ascii_non_ascii() {
        let a = Ascii([0x00, 0xFF]);
        let f = format!("{:?}", a);
        assert_eq!(f, r"[\x00, \xff]");
    }

    #[test]
    fn ascii_always_has_no_newlines() {
        let a = Ascii::new(b"hello!");
        let f = format!("{:#?}", a);
        assert_eq!(f, "[h, e, l, l, o, !]");
    }
}
