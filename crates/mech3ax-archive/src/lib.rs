#![warn(clippy::all, clippy::cargo)]
#![allow(clippy::identity_op)]
mod archive;
mod crc32;

pub use archive::{read_archive, write_archive, Entry, Mode, Version};
