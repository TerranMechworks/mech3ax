use super::Ascii;
use crate::ConversionError;
use std::result::Result as StdResult;

pub type Result<T> = StdResult<T, ConversionError>;

/// Ensures a string is ASCII, or panics. Returns the ASCII bytes.
macro_rules! ensure_str_ascii {
    ($s:ident) => {{
        let s: &str = $s;
        let b = s.as_bytes();
        if !b.is_ascii() {
            panic!("non-ASCII string");
        }
        b
    }};
}

fn is_ascii(v: &[u8]) -> Result<()> {
    // `is_ascii` is optimised, only try and find the first invalid character
    // after it fails
    if !v.is_ascii() {
        for (index, b) in v.iter().enumerate() {
            if b & 0x80 != 0 {
                return Err(ConversionError::NonAscii(index));
            }
        }
        Err(ConversionError::NonAscii(v.len()))
    } else {
        Ok(())
    }
}

pub fn str_from_ascii(v: &[u8]) -> Result<&str> {
    is_ascii(v)?;
    // SAFETY: v is ASCII, and therefore UTF8
    Ok(unsafe { std::str::from_utf8_unchecked(v) })
}

pub fn str_to_ascii(s: &str) -> Result<&[u8]> {
    let v: &[u8] = s.as_bytes();
    is_ascii(v)?;
    Ok(v)
}

pub fn string_from_ascii(v: Vec<u8>) -> Result<String> {
    is_ascii(&v)?;
    // SAFETY: v is ASCII, and therefore UTF8
    Ok(unsafe { String::from_utf8_unchecked(v) })
}

const DEFAULT_NODE_NAME: &[u8] = b"Default_node_name";

impl<const N: usize> Ascii<N> {
    /// Copy from `b` to `self`, leaving at least one byte for zero termination.
    ///
    /// At most `N - 1` bytes will be copied from `b`. Note that this function
    /// does not add zero termination!
    fn copy_with_zero_space(&mut self, b: &[u8]) -> usize {
        let len = b.len();
        let len = if len < N { len } else { N - 1 };
        self.0[..len].copy_from_slice(&b[..len]);
        len
    }

    /// Copy from `b` to `self`, leaving no bytes for zero termination.
    ///
    /// At most `N` bytes will be copied from `b`. Note that this function
    /// does not add zero termination!
    fn copy_without_zero_space(&mut self, b: &[u8]) -> usize {
        let len = b.len();
        let len = if len < N { len } else { N };
        self.0[..len].copy_from_slice(&b[..len]);
        len
    }

    /// Create a new [`Ascii`] buffer set up with the default node name.
    ///
    /// Note that the return value is not zero terminated!
    fn default_node_name() -> Self {
        let mut s = Self::zero();
        s.copy_without_zero_space(DEFAULT_NODE_NAME);
        s
    }

    fn find_first_zero(&self) -> Result<usize> {
        self.0
            .iter()
            .copied()
            .position(|b| b == 0)
            .ok_or(ConversionError::Unterminated)
    }

    /// Converts a string to a zero-terminated, zero-padded fixed length buffer.
    ///
    /// # Panics
    ///
    /// This function panics if the string is not ASCII.
    pub fn from_str_padded(s: &str) -> Self {
        let b = ensure_str_ascii!(s);

        let mut s = Self::zero();
        s.copy_with_zero_space(b);
        // `s` is already zero terminated (being initialized with zeros)

        s
    }

    /// Converts a zero-terminated, zero-padded fixed length buffer to a string.
    pub fn to_str_padded(&self) -> Result<String> {
        let index = self.find_first_zero()?;
        let (s, padding) = self.0.split_at(index);
        if padding.iter().copied().all(|b| b == 0) {
            str_from_ascii(s).map(str::to_string)
        } else {
            Err(ConversionError::PaddingError("zeroes"))
        }
    }

    /// Converts a string to a zero-terminated, default node name-padded fixed
    /// length buffer.
    ///
    /// # Panics
    ///
    /// This function panics if the string is not ASCII.
    pub fn from_str_node_name(s: &str) -> Self {
        let b = ensure_str_ascii!(s);

        let mut s = Self::default_node_name();
        let len = s.copy_with_zero_space(b);
        // zero terminate
        s.0[len] = 0;

        s
    }

    /// Converts a zero-terminated, default node name-padded fixed
    /// length buffer to a string.
    pub fn to_str_node_name(&self) -> Result<String> {
        let index = self.find_first_zero()?;

        let compare = Self::default_node_name();
        let a = &self.0[index + 1..];
        let b = &compare.0[index + 1..];
        if a == b {
            str_from_ascii(&self.0[..index]).map(str::to_string)
        } else {
            Err(ConversionError::PaddingError("node name"))
        }
    }

    /// Converts a string to a zero-terminated, zero-padded fixed length buffer.
    /// The last period (`.`) in the string will be converted to the zero
    /// terminator.
    ///
    /// # Panics
    ///
    /// This function panics if the string is not ASCII.
    pub fn from_str_suffix(s: &str) -> Self {
        let b = ensure_str_ascii!(s);

        let mut s = Self::zero();
        let len = s.copy_without_zero_space(b);
        match s.0[..len].iter_mut().rev().find(|c| **c == b'.') {
            Some(c) => *c = 0,
            None => {
                if len < N {
                    s.0[len] = 0;
                } else {
                    s.0[N - 1] = 0;
                }
            }
        }
        s
    }

    /// Converts a zero-terminated, zero-padded fixed length buffer to a string.
    /// The first zero terminator in the buffer will be converted to a period
    /// (`.`).
    pub fn to_str_suffix(&self) -> Result<String> {
        let mut iter = self.0.iter().copied();
        let suffix_index = iter
            .position(|c| c == 0)
            .ok_or(ConversionError::Unterminated)?;
        let second_zero = iter.position(|c| c == 0);

        // four possibilities:
        // |prefix|0|suffix|0|0,0,..,0| (suffix and padding)
        // |prefix|0|0|0,0,..,0|        (no suffix)
        // |prefix|0|suffix|            (no padding)
        // |prefix|0|                   (no padding or suffix)
        let mut copy = self.to_vec();
        match second_zero {
            Some(relative_padding_index) => {
                // second zero => padding, possibly no suffix
                let index = if relative_padding_index == 0 {
                    // second zero immediately after prefix => no suffix
                    suffix_index
                } else {
                    // second zero noy immediately after prefix => suffix
                    // restore suffix by replacing zero with '.'
                    copy[suffix_index] = b'.';
                    suffix_index + relative_padding_index + 1
                };
                if iter.all(|c| c == 0) {
                    str_from_ascii(&copy[..index]).map(str::to_string)
                } else {
                    Err(ConversionError::PaddingError("zeroes"))
                }
            }
            None => {
                // no second zero => no padding, possibly no suffix
                if suffix_index + 1 < self.0.len() {
                    // zero not at end => suffix, but no padding
                    // restore suffix by replacing zero with '.'
                    copy[suffix_index] = b'.';
                    str_from_ascii(&copy).map(str::to_string)
                } else {
                    // zero at end => no padding, no suffix
                    str_from_ascii(&copy[..suffix_index]).map(str::to_string)
                }
            }
        }
    }

    /// Converts a string to a zero-terminated, garbage-padded fixed length
    /// buffer.
    ///
    /// # Panics
    ///
    /// This function panics if the string is not ASCII.
    pub fn from_str_garbage(s: &str, g: &[u8]) -> Self {
        let b = ensure_str_ascii!(s);

        let mut s = Self::zero();

        // fill buffer with the garbage
        let len = g.len();
        if len < N {
            let start = N - len;
            s.0[start..].copy_from_slice(g);
        } else {
            s.0.copy_from_slice(&g[..N]);
        }
        // fill buffer with the string and zero terminate
        let len = s.copy_with_zero_space(b);
        s.0[len] = 0;

        s
    }

    /// Converts a zero-terminated, garbage-padded fixed length buffer to a
    /// string.
    pub fn to_str_garbage(&self) -> Result<(String, Vec<u8>)> {
        let index = self.find_first_zero()?;
        let pad = Vec::from(&self.0[index + 1..]);
        str_from_ascii(&self.0[..index]).map(|s| (s.to_string(), pad))
    }
}
