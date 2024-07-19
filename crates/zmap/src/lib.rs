#![warn(clippy::all, clippy::cargo)]
#![allow(clippy::identity_op)]
mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::static_assert_size;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct MapHeaderC {
    version: u32, // 00
    unk04: u32,   // 04
    zero08: u32,  // 08
    zero12: u32,  // 12
    zero16: u32,  // 16
    max_x: f32,   // 20
    zero24: u32,  // 24
    max_y: f32,   // 28
}
static_assert_size!(MapHeaderC, 32);
const MAP_VERSION: u32 = 5;

pub use read::read_map;
pub use write::write_map;
