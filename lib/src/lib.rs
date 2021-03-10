#![allow(clippy::identity_op, clippy::cargo_common_metadata)]
pub mod anim;
pub mod archive;
pub mod assert;
mod crc32;
pub mod gamez;
mod image;
pub mod interp;
mod io_ext;
mod light;
pub mod materials;
pub mod mechlib;
mod mesh;
pub mod messages;
pub mod motion;
mod nodes;
pub mod reader;
mod sequence_event;
mod serde;
mod size;
mod string;
pub mod textures;
mod types;

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
