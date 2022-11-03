use std::fmt;

/// A debug list implementation that always prints entries without newlines,
/// regardless of the alternate/pretty formatter settings.
#[must_use = "must eventually call `finish()`"]
struct DebugList<'a, 'b: 'a> {
    fmt: &'a mut fmt::Formatter<'b>,
    result: fmt::Result,
    has_fields: bool,
}

impl<'a, 'b: 'a> DebugList<'a, 'b> {
    pub fn new(fmt: &'a mut fmt::Formatter<'b>) -> Self {
        let result = fmt.write_str("[");
        Self {
            fmt,
            result,
            has_fields: false,
        }
    }

    fn entry(&mut self, entry: &dyn fmt::Debug) {
        self.result = self.result.and_then(|()| {
            if self.has_fields {
                self.fmt.write_str(", ")?;
            }
            entry.fmt(self.fmt)
        });
        self.has_fields = true;
    }

    pub fn entries<D, I>(&mut self, entries: I) -> &mut Self
    where
        D: fmt::Debug,
        I: IntoIterator<Item = D>,
    {
        for entry in entries {
            self.entry(&entry);
        }
        self
    }

    pub fn finish(&mut self) -> fmt::Result {
        self.result.and_then(|()| self.fmt.write_str("]"))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct HexByte(pub u8);

impl fmt::Debug for HexByte {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // lower, unprefixed hex
        write!(f, "{:02x}", self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct AsciiByte(pub u8);

impl fmt::Debug for AsciiByte {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let escaped = std::ascii::escape_default(self.0);
        write!(f, "{}", escaped)
    }
}

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

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Ascii<const N: usize>(pub [u8; N]);

impl<const N: usize> Ascii<N> {
    #[inline]
    pub const fn new() -> Self {
        Self([0u8; N])
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
    use super::{Ascii, Zeros};

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

    #[test]
    fn ascii_all_ascii() {
        let a = Ascii(*b"hello!");
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
        let a = Ascii(*b"hello!");
        let f = format!("{:#?}", a);
        assert_eq!(f, "[h, e, l, l, o, !]");
    }
}
