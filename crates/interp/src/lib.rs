#![warn(clippy::all, clippy::cargo)]
mod read;
mod write;

pub use read::read_interp;
pub use write::write_interp;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_types::{impl_as_bytes, Ascii, Hex};
use time::OffsetDateTime;

const SIGNATURE: Hex<u32> = Hex(0x08971119);
const VERSION: u32 = 7;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct InterpHeaderC {
    signature: Hex<u32>,
    version: u32,
    count: u32,
}
impl_as_bytes!(InterpHeaderC, 12);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct InterpEntryC {
    name: Ascii<120>,
    last_modified: i32,
    start: u32,
}
impl_as_bytes!(InterpEntryC, 128);

fn from_timestamp(ts: i32) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(i64::from(ts))
        .expect("u32 should always be a valid timestamp")
}

fn to_timestamp(dt: OffsetDateTime) -> i32 {
    // Cast safety: truncation simply leads to incorrect timestamp
    dt.unix_timestamp() as _
}

#[cfg(test)]
mod tests;
