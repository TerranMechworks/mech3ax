mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_types::{impl_as_bytes, Bool32, Hex};
pub use read::read_anim;
pub use write::write_anim;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct AnimHeaderC {
    signature: Hex<u32>, // 00
    version: u32,        // 04
    timestamp: u32,      // 08
    count: u32,          // 12
}
impl_as_bytes!(AnimHeaderC, 16);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct AnimInfoC {
    zero00: u32,       // 000
    zero04: u32,       // 004
    zero08: u16,       // 008
    def_count: u16,    // 010
    defs_ptr: u32,     // 012
    script_count: u32, // 016
    scripts_ptr: u32,  // 020
    unk_count: u32,    // 024
    unks_ptr: u32,     // 028
    world_ptr: u32,    // 032
    gravity: f32,      // 036
    unk40: u32,        // 040
    zero44: u32,       // 044
    zero48: u32,       // 048
    zero52: u32,       // 052
    zero56: u32,       // 056
    one60: u32,        // 060
    zero64: u32,       // 064
    zero68: u32,       // 068
    zero72: u32,       // 072
    zero76: u32,       // 076
    zero80: u32,       // 080
    zero84: u32,       // 084
    zero88: u32,       // 088
    zero92: u32,       // 092
    zero96: u32,       // 096
    zero100: u32,      // 100
    zero104: u32,      // 104
}
impl_as_bytes!(AnimInfoC, 108);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct SiScriptC {
    script_name_ptr: u32,  // 00
    object_name_ptr: u32,  // 04
    script_name_len: u8,   // 08
    object_name_len: u8,   // 09
    pad10: u16,            // 10
    spline_interp: Bool32, // 12
    frame_count: u32,      // 16
    script_data_len: u32,  // 20
    script_data_ptr: u32,  // 24
}
impl_as_bytes!(SiScriptC, 28);
