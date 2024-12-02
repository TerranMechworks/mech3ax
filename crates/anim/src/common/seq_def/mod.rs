pub(crate) mod lg;
pub(crate) mod pm;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_types::{impl_as_bytes, Ascii, Zeros};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Default)]
#[repr(C)]
pub(crate) struct SeqDefInfoC {
    pub(crate) name: Ascii<32>,   // 00
    pub(crate) flags: u32,        // 32
    pub(crate) zero36: Zeros<20>, // 36
    pub(crate) pointer: u32,      // 56
    pub(crate) size: u32,         // 60
}
impl_as_bytes!(SeqDefInfoC, 64);

pub(crate) const RESET_SEQUENCE: Ascii<32> =
    Ascii::new(b"RESET_SEQUENCE\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0");
