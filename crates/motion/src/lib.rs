#![warn(clippy::all, clippy::cargo)]
mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_types::{bitflags, impl_as_bytes};
pub use read::read_motion;
pub use write::write_motion;

const VERSION: u32 = 4;

bitflags! {
    struct MotionFlags: u32 {
        // const SCALE = 1 << 1; // never in motion.zbd
        const ROTATION = 1 << 2;
        const TRANSLATION = 1 << 3;
    }
}

impl MotionFlags {
    pub const DEFAULT: Self =
        Self::from_bits_truncate(Self::ROTATION.bits() | Self::TRANSLATION.bits());
}

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct MotionHeaderC {
    version: u32,     // 00
    loop_time: f32,   // 04
    frame_count: u32, // 08
    part_count: u32,  // 12
    unk16: f32,       // 16
    unk20: f32,       // 20
}
impl_as_bytes!(MotionHeaderC, 24);
