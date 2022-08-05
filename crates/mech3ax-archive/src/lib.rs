#![warn(clippy::all, clippy::cargo)]
#![allow(clippy::identity_op)]
mod archive;

use ::serde::{Deserialize, Serialize};
use mech3ax_common::serde::base64;

pub use archive::{read_archive, write_archive};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Mode {
    Reader,
    Sounds,
    Motion,
    ReaderBypass,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Version {
    One,
    Two(Mode),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArchiveEntry {
    pub name: String,
    #[serde(with = "base64")]
    pub garbage: Vec<u8>,
}
