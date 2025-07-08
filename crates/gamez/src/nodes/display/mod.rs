mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::Color;
use mech3ax_types::{impl_as_bytes, AsBytes as _, Offsets};
pub(crate) use read::read;
pub(crate) use write::write;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct DisplayC {
    origin_x: u32,      // 00
    origin_y: u32,      // 04
    resolution_x: u32,  // 08
    resolution_y: u32,  // 12
    clear_color: Color, // 16
}
impl_as_bytes!(DisplayC, 28);

pub(crate) const fn size() -> u32 {
    DisplayC::SIZE
}
