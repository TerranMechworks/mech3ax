use mech3ax_debug::Ascii;

#[derive(Debug)]
pub enum ConversionError {
    NonAscii(usize),
    PaddingError(String),
    Unterminated,
}

fn from_ascii(v: &[u8]) -> Result<&str, usize> {
    if !v.is_ascii() {
        // is_ascii is optimised, only try and find the invalid character after it
        for (index, b) in v.iter().enumerate() {
            if b & 0x80 != 0 {
                return Err(index);
            }
        }
    }
    // SAFETY: v is ASCII, and therefore UTF8
    Ok(unsafe { std::str::from_utf8_unchecked(v) })
}

pub fn bytes_to_c(bytes: &[u8], fill: &mut [u8]) {
    let mut buf = Vec::from(bytes);
    buf.resize(fill.len(), 0);
    fill.copy_from_slice(&buf);
}

const DEFAULT_NODE_NAME: &str = "Default_node_name";

pub fn str_to_c_padded<const N: usize>(str: impl Into<String>, fill: &mut Ascii<N>) {
    let mut buf = str.into().into_bytes();
    if !buf.is_ascii() {
        panic!("Non-ASCII string");
    }
    buf.resize(N - 1, 0);
    buf.push(0);
    fill.0.copy_from_slice(&buf);
}

pub fn str_to_c_node_name<const N: usize>(str: impl Into<String>, fill: &mut Ascii<N>) {
    let mut buf = Vec::from(DEFAULT_NODE_NAME.as_bytes());
    buf.resize(N, 0);
    {
        let mut buf2 = str.into().into_bytes();
        if !buf2.is_ascii() {
            panic!("Non-ASCII string");
        }
        if buf2.len() >= N {
            buf2.resize(N - 1, 0);
        }
        let (left, right) = buf.split_at_mut(buf2.len());
        left.copy_from_slice(&buf2);
        right[0] = 0;
    }
    fill.0.copy_from_slice(&buf);
}

pub fn str_to_c_suffix<const N: usize>(str: impl Into<String>, fill: &mut Ascii<N>) {
    let mut buf = str.into().into_bytes();
    if !buf.is_ascii() {
        panic!("Non-ASCII string");
    }
    buf.resize(N, 0);
    // does not have to be zero-terminated (if a '.' is in the filename, but easiest
    // to gloss over that)
    if let Some(pos) = buf.iter().position(|&c| c == 46) {
        buf[pos] = 0;
    }
    fill.0.copy_from_slice(&buf);
}

pub fn str_to_c_partition<const N: usize>(str: impl Into<String>, pad: &[u8], fill: &mut Ascii<N>) {
    assert!(pad.len() < N, "padding overflows buffer");
    let mut buf = [0u8; N];
    // fill buf with the padding first
    {
        let (_, right) = buf.split_at_mut(N - pad.len());
        right.copy_from_slice(pad);
    }
    // fill buf with the string
    {
        let mut buf2 = str.into().into_bytes();
        if !buf2.is_ascii() {
            panic!("Non-ASCII string");
        }
        if buf2.len() >= N {
            buf2.resize(N - 1, 0);
        }
        let (left, right) = buf.split_at_mut(buf2.len());
        left.copy_from_slice(&buf2);
        right[0] = 0;
    }
    fill.copy_from(&buf);
}

pub fn str_from_c_padded<const N: usize>(buf: &Ascii<N>) -> Result<String, ConversionError> {
    if let Some(zero) = buf.0.iter().position(|&c| c == 0) {
        if buf.0[zero..].iter().any(|&c| c != 0) {
            Err(ConversionError::PaddingError("zeroes".to_owned()))
        } else {
            str_from_c_sized(&buf.0[..zero])
        }
    } else {
        Err(ConversionError::Unterminated)
    }
}

pub fn str_from_c_sized(buf: &[u8]) -> Result<String, ConversionError> {
    match from_ascii(buf) {
        Ok(str) => Ok(str.to_owned()),
        Err(index) => Err(ConversionError::NonAscii(index)),
    }
}

pub fn str_from_c_node_name<const N: usize>(buf: &Ascii<N>) -> Result<String, ConversionError> {
    let mut compare = Vec::from(DEFAULT_NODE_NAME.as_bytes());
    compare.resize(N, 0);

    if let Some(zero) = buf.0.iter().position(|&c| c == 0) {
        if buf
            .0
            .iter()
            .zip(compare.iter())
            .skip(zero + 1)
            .any(|(&c, &d)| c != d)
        {
            Err(ConversionError::PaddingError("node name".to_owned()))
        } else {
            str_from_c_sized(&buf.0[..zero])
        }
    } else {
        Err(ConversionError::Unterminated)
    }
}

pub fn str_from_c_suffix<const N: usize>(buf: &Ascii<N>) -> Result<String, ConversionError> {
    let mut iter = buf.0.iter();
    let pos1 = iter.position(|&c| c == 0);
    let pos2 = iter.position(|&c| c == 0);

    let mut copy = Vec::from(buf.0);
    match (pos1, pos2) {
        (Some(zero1), Some(zero2)) => {
            let zero = if zero2 == 0 {
                // no suffix
                zero1
            } else {
                // restore suffix by replacing zero with '.'
                copy[zero1] = 46;
                zero1 + zero2 + 1
            };
            if buf.0[zero..].iter().any(|&c| c != 0) {
                Err(ConversionError::PaddingError("zeroes".to_owned()))
            } else {
                str_from_c_sized(&copy[..zero])
            }
        }
        (Some(zero1), None) => {
            // restore suffix by replacing zero with '.'
            copy[zero1] = 46;
            // no padding/cut off
            str_from_c_sized(&copy)
        }
        _ => Err(ConversionError::Unterminated),
    }
}

pub fn str_from_c_partition<const N: usize>(
    buf: &Ascii<N>,
) -> Result<(String, Vec<u8>), ConversionError> {
    if let Some(zero) = buf.0.iter().position(|&c| c == 0) {
        let pad = Vec::from(&buf.0[zero + 1..]);
        match from_ascii(&buf.0[..zero]) {
            Ok(str) => Ok((str.to_owned(), pad)),
            Err(index) => Err(ConversionError::NonAscii(index)),
        }
    } else {
        Err(ConversionError::Unterminated)
    }
}

#[cfg(test)]
mod tests;
