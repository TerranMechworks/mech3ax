pub fn str_to_c_padded<S>(str: S, fill: &mut [u8])
where
    S: Into<String>,
{
    let mut bytes = str.into().into_bytes();
    bytes.resize(fill.len() - 1, 0);
    bytes.push(0);
    fill.copy_from_slice(&bytes);
}

pub fn bytes_to_c(bytes: Vec<u8>, fill: &mut [u8]) {
    let mut copy = bytes.clone();
    copy.resize(fill.len(), 0);
    fill.copy_from_slice(&copy);
}

pub enum ConversionError {
    Utf8(std::str::Utf8Error),
    PaddingError(String),
}

pub fn str_from_c_padded(buf: &[u8]) -> Result<String, ConversionError> {
    let len = buf.len();
    let zero = if let Some(zero) = buf.iter().position(|&c| c == 0) {
        if buf[zero..len].iter().any(|&c| c != 0) {
            return Err(ConversionError::PaddingError("zeroes".to_owned()));
        }
        zero
    } else {
        len
    };
    match std::str::from_utf8(&buf[0..zero]) {
        Ok(str) => Ok(str.to_owned()),
        Err(e) => Err(ConversionError::Utf8(e)),
    }
}

pub fn str_from_c_sized(buf: &[u8]) -> Result<String, ConversionError> {
    match std::str::from_utf8(&buf) {
        Ok(str) => Ok(str.to_owned()),
        Err(err) => Err(crate::string::ConversionError::Utf8(err)),
    }
}
