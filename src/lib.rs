#![warn(clippy::all, clippy::cargo)]
#![allow(clippy::identity_op, clippy::multiple_crate_versions)]
pub mod anim;
pub mod archive;
mod assert;
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

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    Assert(assert::AssertionError),
    PeLite(pelite::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::IO(err)
    }
}

impl From<assert::AssertionError> for Error {
    fn from(err: assert::AssertionError) -> Self {
        Self::Assert(err)
    }
}

impl From<pelite::Error> for Error {
    fn from(err: pelite::Error) -> Self {
        Self::PeLite(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
pub type CountingReader<R> = io_ext::CountingReader<R>;
