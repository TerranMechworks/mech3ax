mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_types::{impl_as_bytes, Hex};
pub use read::read_anim;
pub use write::write_anim;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct AnimHeaderC {
    signature: Hex<u32>, // 00
    version: u32,        // 04
}
impl_as_bytes!(AnimHeaderC, 8);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct AnimInfoC {
    zero00: u32,    // 00
    zero04: u32,    // 04
    zero08: u16,    // 08
    def_count: u16, // 10
    defs_ptr: u32,  // 12
    msg_count: u32, // 16
    msgs_ptr: u32,  // 20
    world_ptr: u32, // 24
    gravity: f32,   // 28
    zero32: u32,    // 32
    zero36: u32,    // 36
    zero40: u32,    // 40
    zero44: u32,    // 44
    zero48: u32,    // 48
    zero52: u32,    // 52
    zero56: u32,    // 56
    one60: u32,     // 60
    zero64: u32,    // 64
}
impl_as_bytes!(AnimInfoC, 68);
