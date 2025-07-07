use crate::assert::AssertionError;
use std::fmt;

#[derive(Debug, Clone)]
pub enum PeError {
    Underflow {
        section: String,
        value: u32,
        bound: u32,
    },
    Overflow {
        section: String,
        value: u32,
        bound: u32,
    },
    TryFrom(std::num::TryFromIntError),
    ReadOutOfBounds(usize),
}

impl fmt::Display for PeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Underflow {
                section,
                value,
                bound,
            } => write!(
                f,
                "address {value} underflows bound {bound} in section {section}"
            ),
            Self::Overflow {
                section,
                value,
                bound,
            } => write!(
                f,
                "address {value} overflows bound {bound} in section {section}"
            ),
            Self::TryFrom(e) => e.fmt(f),
            Self::ReadOutOfBounds(offset) => {
                write!(f, "Offset {offset} would cause out-of-bounds read")
            }
        }
    }
}

impl std::error::Error for PeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::TryFrom(e) => Some(e),
            Self::Underflow { .. } | Self::Overflow { .. } | Self::ReadOutOfBounds(_) => None,
        }
    }
}

impl From<std::num::TryFromIntError> for PeError {
    #[inline]
    fn from(e: std::num::TryFromIntError) -> Self {
        Self::TryFrom(e)
    }
}

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    Assert(AssertionError),
    PeError(PeError),
    InvalidAlphaChannel {
        name: String,
        expected: String,
        actual: String,
    },
    InvalidImageFormat {
        name: String,
        color: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(e) => e.fmt(f),
            Self::Assert(e) => e.fmt(f),
            Self::PeError(e) => e.fmt(f),
            Self::InvalidAlphaChannel {
                name,
                expected,
                actual,
            } => write!(
                f,
                "unexpected alpha channel for `{name}` (expected {expected} alpha, found {actual})"
            ),
            Self::InvalidImageFormat { name, color } => write!(
                f,
                "unexpected image format for `{name}` ({color} is not supported)"
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IO(e) => Some(e),
            Self::Assert(e) => Some(e),
            Self::PeError(e) => Some(e),
            Self::InvalidAlphaChannel { .. } | Self::InvalidImageFormat { .. } => None,
        }
    }
}

impl From<std::io::Error> for Error {
    #[inline]
    fn from(e: std::io::Error) -> Self {
        Self::IO(e)
    }
}

impl From<AssertionError> for Error {
    #[inline]
    fn from(e: AssertionError) -> Self {
        Self::Assert(e)
    }
}

impl From<PeError> for Error {
    #[inline]
    fn from(e: PeError) -> Self {
        Self::PeError(e)
    }
}

impl From<String> for Error {
    #[inline]
    fn from(value: String) -> Self {
        Self::Assert(AssertionError(value))
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
