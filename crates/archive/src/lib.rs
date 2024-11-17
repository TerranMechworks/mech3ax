#![warn(clippy::all, clippy::cargo)]
#![allow(clippy::identity_op)]
mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_types::{impl_as_bytes, Ascii, Hex};
pub use read::read_archive;
use std::fmt;
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

/// This exist, because using a single u64 would be unaligned.
#[derive(Clone, Copy, NoUninit, AnyBitPattern, PartialEq)]
#[repr(C)]
struct FiletimeC {
    filetime_lo: u32,
    filetime_hi: u32,
}
impl_as_bytes!(FiletimeC, 8);

impl FiletimeC {
    pub(crate) fn as_u64(&self) -> u64 {
        (u64::from(self.filetime_hi) << 32) | (u64::from(self.filetime_lo) << 0)
    }

    pub(crate) fn from_u64(filetime: u64) -> Self {
        let filetime_hi = ((filetime >> 32) & 0xFFFF_FFFF) as u32;
        let filetime_lo = ((filetime >> 0) & 0xFFFF_FFFF) as u32;
        Self {
            filetime_lo,
            filetime_hi,
        }
    }
}

impl fmt::Debug for FiletimeC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let filetime = self.as_u64();
        <u64 as fmt::Debug>::fmt(&filetime, f)
    }
}

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct TableEntryC {
    start: u32,          // 00
    length: u32,         // 04
    name: Ascii<64>,     // 08
    flags: u32,          // 72
    comment: Ascii<64>,  // 76
    filetime: FiletimeC, // 140
}
impl_as_bytes!(TableEntryC, 148);

#[cfg(test)]
mod tests;
