#![warn(clippy::all, clippy::cargo)]
mod read;
mod write;

pub use read::read_reader;
pub use write::write_reader;

const INT: u32 = 1;
const FLOAT: u32 = 2;
const STRING: u32 = 3;
const LIST: u32 = 4;
