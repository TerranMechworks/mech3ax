pub fn bytes_to_c(bytes: &[u8], fill: &mut [u8]) {
    let mut buf = Vec::from(bytes);
    buf.resize(fill.len(), 0);
    fill.copy_from_slice(&buf);
}

const DEFAULT_NODE_NAME: &'static str = "Default_node_name";

pub fn str_to_c_padded<S>(str: S, fill: &mut [u8])
where
    S: Into<String>,
{
    let mut buf = str.into().into_bytes();
    buf.resize(fill.len() - 1, 0);
    buf.push(0);
    fill.copy_from_slice(&buf);
}

pub fn str_to_c_node_name<S>(str: S, fill: &mut [u8])
where
    S: Into<String>,
{
    let mut buf = Vec::from(DEFAULT_NODE_NAME.as_bytes());
    buf.resize(fill.len(), 0);
    {
        let mut buf2 = str.into().into_bytes();
        if buf2.len() >= fill.len() {
            buf2.resize(fill.len() - 1, 0);
        }
        let (left, right) = buf.split_at_mut(buf2.len());
        left.copy_from_slice(&buf2);
        right[0] = 0;
    }
    fill.copy_from_slice(&buf);
}

pub fn str_to_c_suffix<S>(str: S, fill: &mut [u8])
where
    S: Into<String>,
{
    let mut buf = str.into().into_bytes();
    buf.resize(fill.len(), 0);
    // does not have to be zero-terminated (if a '.' is in the filename, but easiest
    // to gloss over that)
    if let Some(pos) = buf.iter().position(|&c| c == 46) {
        buf[pos] = 0;
    }
    fill.copy_from_slice(&buf);
}

#[derive(Debug)]
pub enum ConversionError {
    Utf8(std::str::Utf8Error),
    PaddingError(String),
    Unterminated,
}

pub fn str_from_c_padded(buf: &[u8]) -> Result<String, ConversionError> {
    let len = buf.len();
    if let Some(zero) = buf.iter().position(|&c| c == 0) {
        if buf[zero..len].iter().any(|&c| c != 0) {
            Err(ConversionError::PaddingError("zeroes".to_owned()))
        } else {
            match std::str::from_utf8(&buf[0..zero]) {
                Ok(str) => Ok(str.to_owned()),
                Err(e) => Err(ConversionError::Utf8(e)),
            }
        }
    } else {
        Err(ConversionError::Unterminated)
    }
}

pub fn str_from_c_sized(buf: &[u8]) -> Result<String, ConversionError> {
    match std::str::from_utf8(&buf) {
        Ok(str) => Ok(str.to_owned()),
        Err(err) => Err(crate::string::ConversionError::Utf8(err)),
    }
}

pub fn str_from_c_node_name(buf: &[u8]) -> Result<String, ConversionError> {
    let len = buf.len();
    let mut compare = Vec::from(DEFAULT_NODE_NAME.as_bytes());
    compare.resize(len, 0);

    if let Some(zero) = buf.iter().position(|&c| c == 0) {
        if buf
            .iter()
            .zip(compare.iter())
            .skip(zero + 1)
            .any(|(&c, &d)| c != d)
        {
            Err(ConversionError::PaddingError("node name".to_owned()))
        } else {
            match std::str::from_utf8(&buf[0..zero]) {
                Ok(str) => Ok(str.to_owned()),
                Err(e) => Err(ConversionError::Utf8(e)),
            }
        }
    } else {
        Err(ConversionError::Unterminated)
    }
}

pub fn str_from_c_suffix(buf: &[u8]) -> Result<String, ConversionError> {
    let len = buf.len();
    let mut iter = buf.iter();
    let pos1 = iter.position(|&c| c == 0);
    let pos2 = iter.position(|&c| c == 0);

    let mut copy = Vec::from(buf);
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
            if buf[zero..len].iter().any(|&c| c != 0) {
                Err(ConversionError::PaddingError("zeroes".to_owned()))
            } else {
                match std::str::from_utf8(&copy[0..zero]) {
                    Ok(str) => Ok(str.to_owned()),
                    Err(e) => Err(ConversionError::Utf8(e)),
                }
            }
        }
        (Some(zero1), None) => {
            // restore suffix by replacing zero with '.'
            copy[zero1] = 46;
            // no padding/cut off
            match std::str::from_utf8(&copy) {
                Ok(str) => Ok(str.to_owned()),
                Err(e) => Err(ConversionError::Utf8(e)),
            }
        }
        _ => return Err(ConversionError::Unterminated),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use matches::assert_matches;

    #[test]
    fn str_from_c_padded_with_zeros() {
        let result = str_from_c_padded("spam eggs\0\0\0\0".to_owned().as_bytes()).unwrap();
        assert_eq!(result, "spam eggs");
    }

    #[test]
    fn str_to_c_padded_with_zeros() {
        let mut buf = [0; 13];
        str_to_c_padded("spam eggs", &mut buf);
        assert_eq!(buf, "spam eggs\0\0\0\0".to_owned().as_bytes());
    }

    #[test]
    fn str_from_c_padded_at_length() {
        let err = str_from_c_padded("spam eggs".to_owned().as_bytes()).unwrap_err();
        assert_matches!(err, ConversionError::Unterminated);
    }

    #[test]
    fn str_to_c_padded_at_length() {
        let mut buf = [0; 9];
        str_to_c_padded("spam eggs", &mut buf);
        assert_eq!(buf, "spam egg\0".to_owned().as_bytes());
    }

    #[test]
    fn str_from_c_padded_with_non_zero() {
        let err = str_from_c_padded("spam eggs\0ham".to_owned().as_bytes()).unwrap_err();
        assert_matches!(err, ConversionError::PaddingError(_));
    }

    #[test]
    fn str_from_c_node_name_with_node_name() {
        let result =
            str_from_c_node_name("foo bar\0node_name\0\0\0".to_owned().as_bytes()).unwrap();
        assert_eq!(result, "foo bar");
    }

    #[test]
    fn str_to_c_node_name_with_node_name() {
        let mut buf = [0; 20];
        str_to_c_node_name("foo bar", &mut buf);
        assert_eq!(buf, "foo bar\0node_name\0\0\0".to_owned().as_bytes());
    }

    #[test]
    fn str_from_c_node_name_at_length() {
        let err = str_from_c_node_name("spam eggs".to_owned().as_bytes()).unwrap_err();
        assert_matches!(err, ConversionError::Unterminated);
    }

    #[test]
    fn str_to_c_node_name_at_length() {
        let mut buf = [0; 9];
        str_to_c_node_name("spam eggs", &mut buf);
        assert_eq!(buf, "spam egg\0".to_owned().as_bytes());
    }

    #[test]
    fn str_from_c_node_name_with_zeros() {
        let err = str_from_c_node_name("spam eggs\0\0\0\0".to_owned().as_bytes()).unwrap_err();
        assert_matches!(err, ConversionError::PaddingError(_));
    }

    #[test]
    fn str_from_c_suffix_with_suffix() {
        let result = str_from_c_suffix("foo bar\0tif\0".to_owned().as_bytes()).unwrap();
        assert_eq!(result, "foo bar.tif");
    }

    #[test]
    fn str_to_c_suffix_with_suffix() {
        let mut buf = [0; 12];
        str_to_c_suffix("foo bar.tif", &mut buf);
        assert_eq!(buf, "foo bar\0tif\0".to_owned().as_bytes());
    }

    #[test]
    fn str_from_c_suffix_no_suffix() {
        let result = str_from_c_suffix("foo bar\0\0".to_owned().as_bytes()).unwrap();
        assert_eq!(result, "foo bar");
    }

    #[test]
    fn str_to_c_suffix_no_suffix() {
        let mut buf = [0; 9];
        str_to_c_suffix("foo bar", &mut buf);
        assert_eq!(buf, "foo bar\0\0".to_owned().as_bytes());
    }

    #[test]
    fn str_from_c_suffix_completely_unterminated() {
        let err = str_from_c_suffix("foo bar".to_owned().as_bytes()).unwrap_err();
        assert_matches!(err, ConversionError::Unterminated);
    }

    #[test]
    fn str_from_c_suffix_with_suffix_unterminated() {
        let result = str_from_c_suffix("foo bar\0tif".to_owned().as_bytes()).unwrap();
        assert_eq!(result, "foo bar.tif");
    }

    #[test]
    fn str_from_c_suffix_with_suffix_with_non_zeros() {
        let err = str_from_c_suffix("foo bar\0tif\0ham\0".to_owned().as_bytes()).unwrap_err();
        assert_matches!(err, ConversionError::PaddingError(_));
    }
}
