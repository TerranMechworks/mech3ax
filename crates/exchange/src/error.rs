use serde::{de, ser};
use std::fmt;

/// The detailed cause of an error.
#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorCode {
    // --- General ---
    /// A custom error message.
    ///
    /// This is how serde errors are reported.
    Custom(String),
    /// An error occurred during an I/O operation.
    IO(std::io::Error),
    /// The data type is not supported by the serializer or deserializer.
    UnsupportedType,

    // --- Deserializers ---
    /// The type specifier is invalid.
    InvalidType,
    /// The type is unexpected.
    UnexpectedType {
        expected: String,
        actual: String,
    },
    InvalidUtf8(std::string::FromUtf8Error),
    InvalidVariant,

    // --- Writers ---
    /// A sequence must have a length to be serialized.
    SequenceMustHaveLength,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // General
            ErrorCode::Custom(s) => write!(f, "{}", s),
            ErrorCode::IO(e) => fmt::Display::fmt(e, f),
            ErrorCode::UnsupportedType => f.write_str("unsupported type"),
            // Deserializers
            ErrorCode::InvalidType => f.write_str("invalid type"),
            ErrorCode::UnexpectedType { expected, actual } => {
                write!(f, "expected {}, found {}", expected, actual)
            }
            ErrorCode::InvalidUtf8(e) => fmt::Display::fmt(e, f),
            ErrorCode::InvalidVariant => f.write_str("invalid variant"),
            // Writers
            ErrorCode::SequenceMustHaveLength => f.write_str("sequence must have a known length"),
        }
    }
}

/// This type represents all possible errors that can occur when serializing or
/// deserializing binary zlisp data.
#[derive(Debug)]
pub struct Error(Box<ErrorCode>);

/// A specialized [Result](std::result::Result) type for serialization or
/// deserialization operations.
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Construct a new error.
    #[cold]
    pub fn new(code: ErrorCode) -> Self {
        Self(Box::new(code))
    }

    /// The error code.
    pub const fn code(&self) -> &ErrorCode {
        &self.0
    }

    /// Construct a new IO error.
    #[cold]
    pub fn io(e: std::io::Error) -> Self {
        Self::new(ErrorCode::IO(e))
    }

    /// Construct a new IO error.
    #[cold]
    pub fn unexpected_type<E, A>(expected: E, actual: A) -> Self
    where
        E: Into<String>,
        A: Into<String>,
    {
        Self::new(ErrorCode::UnexpectedType {
            expected: expected.into(),
            actual: actual.into(),
        })
    }

    fn custom_ser<T: fmt::Display>(msg: T) -> Self {
        Self::new(ErrorCode::Custom(msg.to_string()))
    }

    fn custom_de<T: fmt::Display>(msg: T) -> Self {
        Self::new(ErrorCode::Custom(msg.to_string()))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::custom_ser(msg)
    }
}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::custom_de(msg)
    }
}

impl de::StdError for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &*self.0 {
            ErrorCode::IO(e) => Some(e),
            ErrorCode::InvalidUtf8(e) => Some(e),
            _ => None,
        }
    }
}
