pub mod archive;
mod assert;
mod image;
pub mod interp;
mod io_ext;
pub mod materials;
pub mod mechlib;
pub mod messages;
pub mod motion;
pub mod reader;
mod serde;
mod size;
mod string;
pub mod textures;

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
