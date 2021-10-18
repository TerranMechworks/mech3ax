#![warn(clippy::all, clippy::cargo)]
pub mod assert;
pub mod io_ext;
pub mod light;
pub mod serde;
pub mod size;
pub mod string;
pub mod types;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Assert(#[from] assert::AssertionError),
    #[error(transparent)]
    PeLite(#[from] pelite::Error),
    #[error("Unexpected alpha channel for \"{name}\" (expected {expected} alpha, found {actual})")]
    InvalidAlphaChannel {
        name: String,
        expected: String,
        actual: String,
    },
    #[error("Unexpected image format for \"{name}\" ({color:?} is not supported)")]
    InvalidImageFormat {
        name: String,
        color: ::image::ColorType,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
pub type CountingReader<R> = io_ext::CountingReader<R>;
