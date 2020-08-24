pub mod archive;
mod assert;
pub mod interp;
mod io_ext;
mod serde;
mod size;
mod string;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    Assert(assert::AssertionError),
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

pub type Result<T> = std::result::Result<T, Error>;
