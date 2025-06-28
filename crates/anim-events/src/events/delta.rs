use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_types::impl_as_bytes;

#[derive(Debug, Clone, Copy, PartialEq, NoUninit, AnyBitPattern)]
#[repr(C)]
pub(crate) struct FloatFromToC {
    pub(crate) from: f32,  // 00
    pub(crate) to: f32,    // 04
    pub(crate) delta: f32, // 08
}
impl_as_bytes!(FloatFromToC, 12);

impl FloatFromToC {
    pub(crate) const DEFAULT: Self = Self {
        from: 0.0,
        to: 0.0,
        delta: 0.0,
    };
}

#[inline]
pub(crate) fn delta(from: f32, to: f32, run_time: f32) -> f32 {
    if to == from || run_time == 0.0 {
        0.0
    } else {
        (to - from) / run_time
    }
}

// #[inline]
// pub(crate) fn dec_f32(value: f32) -> f32 {
//     let dec = u32::from_ne_bytes(value.to_ne_bytes()) - 1;
//     f32::from_ne_bytes(dec.to_ne_bytes())
// }
