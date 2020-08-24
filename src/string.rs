pub fn str_to_c<S>(str: S, fill: &mut [u8])
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

pub fn str_from_c(raw: &[u8]) -> std::result::Result<&str, std::str::Utf8Error> {
    let len = raw.iter().position(|&c| c == b'\0').unwrap_or(raw.len());
    std::str::from_utf8(&raw[0..len])
}
