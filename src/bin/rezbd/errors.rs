use image::error::ImageError;
use mech3rs::Error as MechError;
use serde_json::Error as SerdeError;
use std::io::Error as IOError;
use zip::result::ZipError;

#[derive(Debug)]
pub enum Error {
    IO(IOError),
    Zip(ZipError),
    Serde(SerdeError),
    Mech(MechError),
    Image(ImageError),
}

impl From<IOError> for Error {
    fn from(err: IOError) -> Self {
        Self::IO(err)
    }
}

impl From<ZipError> for Error {
    fn from(err: ZipError) -> Self {
        Self::Zip(err)
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

pub type Result<T> = ::std::result::Result<T, Error>;
