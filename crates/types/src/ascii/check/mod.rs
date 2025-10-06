use super::Ascii;
use crate::ConversionError;

pub fn node_name<const N: usize>(value: &Ascii<N>) -> Result<String, String> {
    value.to_str_node_name().map_err(|e| match e {
        ConversionError::PaddingError(padding) => format!("expected string padding `{}`", padding),
        ConversionError::NonAscii(index) => {
            format!("invalid character at +{}", index)
        }
        ConversionError::Unterminated => "missing zero terminator".to_string(),
    })
}

pub fn suffix<const N: usize>(value: &Ascii<N>) -> Result<String, String> {
    value.to_str_suffix().map_err(|e| match e {
        ConversionError::PaddingError(padding) => format!("expected string padding `{}`", padding),
        ConversionError::NonAscii(index) => {
            format!("invalid character at +{}", index)
        }
        ConversionError::Unterminated => "missing zero terminator".to_string(),
    })
}

fn as_ascii(v: &[u8]) -> Result<String, String> {
    // `is_ascii` is optimised, only try and find the first invalid character
    // after it fails
    if !v.is_ascii() {
        let index = v
            .iter()
            .copied()
            .position(|b| b & 0x80 != 0)
            .unwrap_or(v.len());
        Err(format!("invalid character at +{}", index))
    } else {
        // SAFETY: v is ASCII, and therefore UTF8
        Ok(unsafe { std::str::from_utf8_unchecked(v) }.to_string())
    }
}

fn as_zero(v: &[u8]) -> Option<Vec<u8>> {
    if v.iter().copied().all(|b| b == 0) {
        None
    } else {
        Some(v.to_vec())
    }
}

/// Converts a zero-terminated, potentially garbage-padded fixed length buffer
/// to a string and padding.
pub fn garbage<const N: usize>(v: &Ascii<N>) -> Result<(String, Option<Vec<u8>>), String> {
    let mid =
        v.0.iter()
            .copied()
            .position(|b| b == 0)
            .ok_or("missing zero terminator".to_string())?;

    let name = &v.0[..mid];
    let pad = &v.0[mid + 1..];

    let name = as_ascii(name)?;
    let pad = as_zero(pad);

    Ok((name, pad))
}

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

pub fn make_garbage<const N: usize, P>(n: &str, p: &Option<P>) -> Ascii<N>
where
    P: std::ops::Deref<Target = [u8]>,
{
    let n: &[u8] = ensure_str_ascii!(n);

    let mut s = Ascii::zero();

    if let Some(pad) = p.as_ref() {
        let p: &[u8] = std::ops::Deref::deref(pad);

        // fill buffer with the padding from the back:
        // * if the padding is too long, it will be silently trimmed
        // * if the name has been modified, but the padding hasn't, then the
        //   padding will be overwritten. this makes sense to me, as the
        //   padding/garbage is whatever was left in memory.
        let len = p.len();
        if len >= N {
            s.0.copy_from_slice(&p[..N]);
        } else {
            let start = N - len;
            s.0[start..].copy_from_slice(p);
        }
    }

    // fill buffer with the name from the front:
    // * if the name is too long, it will be silently trimmed
    let len = n.len();
    let len = if len < N { len } else { N - 1 };
    s.0[..len].copy_from_slice(&n[..len]);
    s.0[len] = 0;

    s
}

#[cfg(test)]
mod tests;
