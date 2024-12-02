mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::{Quaternion, Vec3};
use mech3ax_types::{bitflags, impl_as_bytes, Bytes, Maybe};
pub use read::read_si_script_frames;
pub use write::{size_si_script_frames, write_si_script_frames};

bitflags! {
    struct FrameFlags: u32 {
        const TRANSLATE = 1 << 0;
        const ROTATE = 1 << 1;
        const SCALE = 1 << 2;
    }
}

type Flags = Maybe<u32, FrameFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct FrameC {
    flags: Flags,    // 00
    start_time: f32, // 04
    end_time: f32,   // 08
}
impl_as_bytes!(FrameC, 12);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct TranslateDataC {
    base: Vec3,          // 00
    unk: u32,            // 12 f32
    delta: Vec3,         // 16
    spline_x: Bytes<16>, // 28
    spline_y: Bytes<16>, // 44
    spline_z: Bytes<16>, // 60
}
impl_as_bytes!(TranslateDataC, 76);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct RotateDataC {
    base: Quaternion,    // 00
    delta: Vec3,         // 16
    spline_x: Bytes<16>, // 28
    spline_y: Bytes<16>, // 44
    spline_z: Bytes<16>, // 60
}
impl_as_bytes!(RotateDataC, 76);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ScaleDataC {
    base: Vec3,          // 00
    unk: u32,            // 12 f32
    delta: Vec3,         // 16
    spline_x: Bytes<16>, // 28
    spline_y: Bytes<16>, // 44
    spline_z: Bytes<16>, // 60
}
impl_as_bytes!(ScaleDataC, 76);
