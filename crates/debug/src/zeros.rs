use crate::byte_repr::HexByte;
use crate::debug_list::DebugList;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Zeros<const N: usize>(pub [u8; N]);

impl<const N: usize> Zeros<N> {
    #[inline]
    pub const fn new() -> Self {
        Self([0u8; N])
    }
}

impl<const N: usize> fmt::Debug for Zeros<N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.iter().copied().all(|b| b == 0) {
            write!(f, "[0 x {}]", N)
        } else {
            let entries = self.0.iter().copied().map(HexByte);
            DebugList::new(f).entries(entries).finish()
        }
    }
}

impl<const N: usize> From<[u8; N]> for Zeros<N> {
    #[inline]
    fn from(inner: [u8; N]) -> Self {
        Self(inner)
    }
}

#[cfg(test)]
mod tests {
    use super::Zeros;

    #[test]
    fn zeros_all_zero() {
        let z: Zeros<8> = Zeros::new();
        let f = format!("{:?}", z);
        assert_eq!(f, "[0 x 8]");
    }

    #[test]
    fn zeros_non_zero() {
        let z = Zeros([0xfa, 0x11, 0xed]);
        let f = format!("{:?}", z);
        assert_eq!(f, "[fa, 11, ed]");
    }

    #[test]
    fn zeros_always_has_no_newlines() {
        let z = Zeros([0xfa, 0x11, 0xed]);
        let f = format!("{:#?}", z);
        assert_eq!(f, "[fa, 11, ed]");
    }
}
