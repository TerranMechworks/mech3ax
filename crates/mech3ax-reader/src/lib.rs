#![warn(clippy::all, clippy::cargo)]
mod reader;

pub use reader::{read_reader, write_reader};
