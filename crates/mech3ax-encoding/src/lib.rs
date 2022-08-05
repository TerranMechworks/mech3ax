#![warn(clippy::all, clippy::cargo)]

use std::borrow::Cow;

include!(concat!(env!("OUT_DIR"), "/index-windows-1252.rs"));

pub fn windows1252_decode(bytes: &[u8]) -> Cow<str> {
    if bytes.is_ascii() {
        Cow::Borrowed(unsafe { std::str::from_utf8_unchecked(bytes) })
    } else {
        Cow::Owned(
            bytes
                .iter()
                .map(|&byte| WINDOWS1252[byte as usize])
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests;
