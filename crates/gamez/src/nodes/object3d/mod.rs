mod math;
mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::{AffineMatrix, Color, Vec3};
use mech3ax_types::{AsBytes as _, Maybe, Offsets, bitflags, impl_as_bytes};
pub(crate) use read::read;
pub(crate) use write::write;

bitflags! {
    struct Object3dFlags: u32 {
        // const NEEDS_UPDATE = 1 << 0;        // 0x01
        const OPACITY = 1 << 1;             // 0x02
        const COLOR = 1 << 2;               // 0x04
        const TRANSFORM_INITIAL = 1 << 3;   // 0x08
        const USE_MATRIX = 1 << 4;          // 0x10
        const UNK5 = 1 << 5;                // 0x20
        // const UNK6 = 1 << 6;                // 0x40
        // const UNK7 = 1 << 7;                // 0x80
    }
}

type Flags = Maybe<u32, Object3dFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct Object3dC {
    flags: Flags,            // 000
    opacity: f32,            // 004
    color: Color,            // 008
    field020: f32,           // 020
    rotate: Vec3,            // 024
    scale: Vec3,             // 032
    transform: AffineMatrix, // 048
    field096: AffineMatrix,  // 096
}
impl_as_bytes!(Object3dC, 144);

pub(crate) fn size() -> u32 {
    Object3dC::SIZE
}

const SCALE_INITIAL: Vec3 = Vec3 {
    x: 1.0,
    y: 1.0,
    z: 1.0,
};
