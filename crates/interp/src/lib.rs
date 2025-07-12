#![warn(clippy::all, clippy::cargo)]
#![allow(clippy::identity_op)]
mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_types::{Ascii, Hex, impl_as_bytes};
pub use read::read_interp;
pub use write::write_interp;

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
    timestamp: u32,
    start: u32,
}
impl_as_bytes!(InterpEntryC, 128);
