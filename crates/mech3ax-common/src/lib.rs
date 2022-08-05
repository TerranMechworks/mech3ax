#![warn(clippy::all, clippy::cargo)]
pub mod assert;
pub mod io_ext;
pub mod light;
pub mod string;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PeError {
    #[error("Address {value} underflows bound {bound} in section {section}")]
    Underflow {
        section: String,
        value: u32,
        bound: u32,
    },
    #[error("Address {value} overflows bound {bound} in section {section}")]
    Overflow {
        section: String,
        value: u32,
        bound: u32,
    },
    #[error(transparent)]
    TryFrom(#[from] std::num::TryFromIntError),
    #[error("Offset {0} would cause out-of-bounds read")]
    ReadOutOfBounds(usize),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Assert(#[from] assert::AssertionError),
    #[error(transparent)]
    PeError(#[from] PeError),
    #[error("Unexpected alpha channel for \"{name}\" (expected {expected} alpha, found {actual})")]
    InvalidAlphaChannel {
        name: String,
        expected: String,
        actual: String,
    },
    #[error("Unexpected image format for \"{name}\" ({color} is not supported)")]
    InvalidImageFormat { name: String, color: String },
}

pub type Result<T> = std::result::Result<T, Error>;
pub type CountingReader<R> = io_ext::CountingReader<R>;
