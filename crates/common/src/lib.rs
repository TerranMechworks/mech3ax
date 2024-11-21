#![warn(clippy::all, clippy::cargo)]
pub mod assert;
mod errors;
pub mod io_ext;
pub mod light;
pub mod string;

pub use errors::{Error, PeError, Result};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameType {
    MW,
    PM,
    RC,
    CS,
}

impl fmt::Display for GameType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::MW => "MW",
            Self::PM => "PM",
            Self::RC => "RC",
            Self::CS => "CS",
        };
        f.write_str(s)
    }
}
