mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_types::{impl_as_bytes, Ascii, Zeros};
pub(crate) use read::{
    read_reset_state_pg, read_reset_state_pm, read_sequence_defs, ReadEventsMw, ReadEventsPm,
    ReadEventsRc,
};
pub(crate) use write::{
    write_reset_state_pg, write_reset_state_pm, write_sequence_defs, WriteEventsMw, WriteEventsPm,
    WriteEventsRc,
};

#[derive(Debug, Clone, Copy, PartialEq, NoUninit, AnyBitPattern, Default)]
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

const SEQ_ACTIVATION_ON_CALL: u32 = 0x0303;
const SEQ_ACTIVATION_INITIAL: u32 = 0;
