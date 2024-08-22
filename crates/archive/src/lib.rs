#![warn(clippy::all, clippy::cargo)]
#![allow(clippy::identity_op)]
mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_types::{impl_as_bytes, Ascii, Bytes, Hex};
pub use read::read_archive;
pub use write::write_archive;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Reader,
    Sounds,
    Motion,
    ReaderBypass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Version {
    One,
    Two(Mode),
}

const VERSION_ONE: u32 = 1;
const VERSION_TWO: u32 = 2;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct HeaderOneC {
    version: u32, // 0
    count: u32,   // 4
}
impl_as_bytes!(HeaderOneC, 8);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct HeaderTwoC {
    version: u32,       // 0
    count: u32,         // 4
    checksum: Hex<u32>, // 8
}
impl_as_bytes!(HeaderTwoC, 12);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct TableEntryC {
    start: u32,         // 00
    length: u32,        // 04
    name: Ascii<64>,    // 08
    garbage: Bytes<76>, // 72
}
impl_as_bytes!(TableEntryC, 148);
