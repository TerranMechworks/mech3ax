#![warn(clippy::all, clippy::cargo)]
#![allow(clippy::identity_op)]
mod archive;

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
