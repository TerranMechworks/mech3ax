mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::SeqDefState;
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{impl_as_bytes, Ascii, Maybe, Ptr};
pub(crate) use read::{
    read_reset_state_pg, read_reset_state_pm, read_sequence_defs, ReadEventsMw, ReadEventsPm,
    ReadEventsRc,
};
pub(crate) use write::{
    write_reset_state_pg, write_reset_state_pm, write_sequence_defs, WriteEventsMw, WriteEventsPm,
    WriteEventsRc,
};

type State = Maybe<u8, SeqDefState>;

#[derive(Debug, Clone, Copy, PartialEq, NoUninit, AnyBitPattern, Default)]
#[repr(C)]
pub(crate) struct SeqDefInfoC {
    pub(crate) name: Ascii<32>,     // 00
    pub(crate) seq_state: State,    // 32
    pub(crate) reset_state: State,  // 33
    pub(crate) pad34: u16,          // 34
    pub(crate) loop_time: f32,      // 36 guess
    pub(crate) event_time: f32,     // 40 guess
    pub(crate) seq_time: f32,       // 44 guess
    pub(crate) loop_count: u32,     // 48
    pub(crate) curr_event_ptr: u32, // 52
    pub(crate) pointer: Ptr,        // 56
    pub(crate) size: u32,           // 60
}
impl_as_bytes!(SeqDefInfoC, 64);

impl SeqDefInfoC {
    pub(crate) fn make_reset_state(pointer: u32, size: u32) -> Self {
        Self {
            name: RESET_SEQUENCE,
            seq_state: SeqDefState::Initial.maybe(),
            reset_state: SeqDefState::Initial.maybe(),
            pad34: 0,
            loop_time: 0.0,
            event_time: 0.0,
            seq_time: 0.0,
            loop_count: 0,
            curr_event_ptr: 0,
            pointer: Ptr(pointer),
            size,
        }
    }

    pub(super) fn assert_fields(&self, offset: usize) -> Result<()> {
        assert_that!(
            "anim def reset state field 34",
            self.pad34 == 0,
            offset + 34
        )?;
        assert_that!(
            "anim def reset state field 36",
            self.loop_time == 0.0,
            offset + 36
        )?;
        assert_that!(
            "anim def reset state field 40",
            self.event_time == 0.0,
            offset + 40
        )?;
        assert_that!(
            "anim def reset state field 44",
            self.seq_time == 0.0,
            offset + 44
        )?;
        assert_that!(
            "anim def reset state field 48",
            self.loop_count == 0,
            offset + 48
        )?;
        assert_that!(
            "anim def reset state field 52",
            self.curr_event_ptr == 0,
            offset + 52
        )?;
        Ok(())
    }

    pub(crate) fn assert_reset_state(&self, offset: usize) -> Result<(u32, u32)> {
        assert_that!(
            "anim def reset state name",
            self.name == RESET_SEQUENCE,
            offset + 0
        )?;
        assert_that!(
            "anim def reset state seq state",
            self.seq_state == SeqDefState::Initial.maybe(),
            offset + 32
        )?;
        assert_that!(
            "anim def reset state reset state",
            self.reset_state == SeqDefState::Initial.maybe(),
            offset + 33
        )?;

        self.assert_fields(offset)?;
        Ok((self.pointer.0, self.size))
    }
}

const RESET_SEQUENCE: Ascii<32> = Ascii::new(b"RESET_SEQUENCE\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0");
