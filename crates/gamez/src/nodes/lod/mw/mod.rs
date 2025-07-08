mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_types::{impl_as_bytes, Offsets};
pub(crate) use read::read;
pub(crate) use write::write;

pub(crate) fn size() -> u32 {
    use mech3ax_types::AsBytes as _;
    LodMwC::SIZE
}

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct LodMwC {
    field00: i32,       // 00
    range_near_sq: f32, // 04
    range_far: f32,     // 08
    range_far_sq: f32,  // 12
    field16: f32,       // 16
    field20: f32,       // 20
    field24: f32,       // 24
    field28: f32,       // 28
    field32: f32,       // 32
    field36: f32,       // 36
    field40: f32,       // 40
    field44: f32,       // 44
    field48: i32,       // 48
    field52: f32,       // 52
    field56: f32,       // 56
    field60: f32,       // 60
    field64: f32,       // 64
    field68: i32,       // 68
    field72: f32,       // 72
    field76: f32,       // 76
}
impl_as_bytes!(LodMwC, 80);
