mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::{AffineMatrix, Color, Vec3};
use mech3ax_types::{bitflags, impl_as_bytes, Maybe, Offsets};
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
    }
}

type Flags = Maybe<u32, Object3dFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct Object3dPmC {
    flags: Flags,            // 000
    opacity: f32,            // 004
    color: Color,            // 008
    field020: f32,           // 020
    rotate: Vec3,            // 024
    scale: Vec3,             // 032
    transform: AffineMatrix, // 048
    field096: AffineMatrix,  // 096
}
impl_as_bytes!(Object3dPmC, 144);

const SCALE_INITIAL: Vec3 = Vec3 {
    x: 1.0,
    y: 1.0,
    z: 1.0,
};
