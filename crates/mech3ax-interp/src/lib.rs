#![warn(clippy::all, clippy::cargo)]
mod interp;

pub use interp::{read_interp, write_interp};
