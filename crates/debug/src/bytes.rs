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
        Self([0u8; N])
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

#[cfg(test)]
mod tests {
    use super::Bytes;

    #[test]
    fn bytes_all_zero() {
        let z = Bytes([0x00, 0x00, 0x00]);
        let f = format!("{:?}", z);
        assert_eq!(f, "[00, 00, 00]");
    }

    #[test]
    fn bytes_non_zero() {
        let z = Bytes([0xfa, 0x11, 0xed]);
        let f = format!("{:?}", z);
        assert_eq!(f, "[fa, 11, ed]");
    }

    #[test]
    fn bytes_always_has_no_newlines() {
        let z = Bytes([0xfa, 0x11, 0xed]);
        let f = format!("{:#?}", z);
        assert_eq!(f, "[fa, 11, ed]");
    }
}
