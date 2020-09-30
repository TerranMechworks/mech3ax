use image::error::ImageError;
use mech3rs::Error as MechError;
use serde_json::Error as SerdeError;
use std::ffi::NulError;
use std::io::Error as IOError;
use std::str::Utf8Error;

#[derive(Debug)]
pub(crate) enum Error {
    IO(IOError),
    Serde(SerdeError),
    Mech(MechError),
    Image(ImageError),
    Nul(NulError),
    Utf8(Utf8Error),
}

impl From<IOError> for Error {
    fn from(err: IOError) -> Self {
        Self::IO(err)
    }
}

impl From<NulError> for Error {
    fn from(err: NulError) -> Self {
        Self::Nul(err)
    }
}

impl From<MechError> for Error {
    fn from(err: MechError) -> Self {
        Self::Mech(err)
    }
}

impl From<SerdeError> for Error {
    fn from(err: SerdeError) -> Self {
        Self::Serde(err)
    }
}

impl From<ImageError> for Error {
    fn from(err: ImageError) -> Self {
        Self::Image(err)
    }
}

impl From<Utf8Error> for Error {
    fn from(err: Utf8Error) -> Self {
        Self::Utf8(err)
    }
}

pub(crate) type Result<T> = ::std::result::Result<T, Error>;
