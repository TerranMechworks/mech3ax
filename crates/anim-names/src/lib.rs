//! This crate is a giant hack with one purpose: To ensure all animation
//! definitions (anim defs) have nice names while being binary-accurate.
//!
//! The vast majority of anim names and anim root names are properly
//! zero-terminated, but some are not. Previously, to handle this, the padding
//! had to be stored for both values in the anim def. That shifts the burden to
//! consumers of the data. Instead, I decided to store this information in the
//! code instead.
pub mod mw;
pub mod pm;
pub mod rc;

macro_rules! fwd {
    ($name:ident, $size:literal, $index:expr, $table:expr) => {
        pub fn $name(name: &[u8; $size]) -> Option<(u32, &'static str)> {
            let hash = fxhash::hash32(name);
            $index.binary_search(&hash).ok().map(|index| {
                let (bytes, string) = $table[index];
                assert_eq!(bytes, name);
                (hash, string)
            })
        }
    };
}
pub(crate) use fwd;

macro_rules! rev {
    ($name:ident, $size:literal, $index:expr, $table:expr) => {
        pub fn $name(hash: u32, name: &str) -> Option<&'static [u8; $size]> {
            $index.binary_search(&hash).ok().and_then(|index| {
                let (bytes, string) = $table[index];
                // guard against updating the string but not the hash
                if name == string { Some(bytes) } else { None }
            })
        }
    };
}
pub(crate) use rev;

#[cfg(test)]
mod tests;
