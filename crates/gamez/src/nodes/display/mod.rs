mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::Color;
use mech3ax_types::{impl_as_bytes, Offsets};

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

mod size {
    pub(crate) fn size() -> u32 {
        use mech3ax_types::AsBytes as _;
        super::DisplayC::SIZE
    }
}

pub(crate) mod rc {
    pub(crate) use super::read::read;
    pub(crate) use super::size::size;
    pub(crate) use super::write::write;
}

pub(crate) mod mw {
    pub(crate) use super::read::read;
    pub(crate) use super::size::size;
    pub(crate) use super::write::write;
}
