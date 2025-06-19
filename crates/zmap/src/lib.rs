#![warn(clippy::all, clippy::cargo)]
#![allow(clippy::identity_op)]
mod read;
mod write;

pub use read::read_map;
pub use write::write_map;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::Vec3;
use mech3ax_types::impl_as_bytes;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct MapHeaderC {
    version: u32, // 00
    unk04: u32,   // 04
    min: Vec3,    // 08
    max: Vec3,    // 20
}
impl_as_bytes!(MapHeaderC, 32);
const MAP_VERSION: u32 = 5;
