use super::{SeqDefInfoC, RESET_SEQUENCE, SEQ_ACTIVATION_INITIAL, SEQ_ACTIVATION_ON_CALL};
use log::trace;
use mech3ax_api_types::anim::events::Event;
use mech3ax_api_types::anim::{AnimDef, ResetState, SeqActivation, SeqDef, SiScript};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{assert_that, assert_with_msg, Result};
use std::io::Read;

pub(crate) trait ReadEvents<R: Read> {
    fn read<'a>(
        &'a mut self,
        read: &'a mut CountingReader<R>,
        length: u32,
        anim_def: &'a AnimDef,
    ) -> Result<Vec<Event>>;
}

#[derive(Debug)]
pub(crate) struct ReadEventsMw<'a> {
    pub(crate) scripts: &'a mut Vec<SiScript>,
}

impl<R: Read> ReadEvents<R> for ReadEventsMw<'_> {
    fn read(
        &mut self,
        read: &mut CountingReader<R>,
        length: u32,
        anim_def: &AnimDef,
    ) -> Result<Vec<Event>> {
        mech3ax_anim_events::mw::read_events(read, length, anim_def, self.scripts)
    }
}

#[derive(Debug)]
pub(crate) struct ReadEventsPm;

impl<R: Read> ReadEvents<R> for ReadEventsPm {
    fn read(
        &mut self,
        read: &mut CountingReader<R>,
        length: u32,
        anim_def: &AnimDef,
    ) -> Result<Vec<Event>> {
        mech3ax_anim_events::pm::read_events(read, length, anim_def)
    }
}

#[derive(Debug)]
pub(crate) struct ReadEventsRc<'a> {
    pub(crate) scripts: &'a mut Vec<SiScript>,
}

impl<R: Read> ReadEvents<R> for ReadEventsRc<'_> {
    fn read(
        &mut self,
        read: &mut CountingReader<R>,
        length: u32,
        anim_def: &AnimDef,
    ) -> Result<Vec<Event>> {
        mech3ax_anim_events::rc::read_events(read, length, anim_def, self.scripts)
    }
}

pub(crate) fn read_sequence_defs<R, RE>(
    read: &mut CountingReader<R>,
    anim_def: &AnimDef,
    count: u8,
    mut read_events: RE,
) -> Result<Vec<SeqDef>>
where
    R: Read,
    RE: ReadEvents<R>,
{
    (0..count)
        .map(|index| {
            trace!("Reading sequence definition {}", index);
            let seq_def: SeqDefInfoC = read.read_struct()?;

            let name = assert_utf8("seq def name", read.prev + 0, || {
                seq_def.name.to_str_padded()
            })?;

            let activation = match seq_def.flags {
                SEQ_ACTIVATION_INITIAL => SeqActivation::Initial,
                SEQ_ACTIVATION_ON_CALL => SeqActivation::OnCall,
                _ => {
                    return Err(assert_with_msg!(
                        "Expected valid seq def flags, but was 0x{:08X} (at {})",
                        seq_def.flags,
                        read.prev + 32
                    ))
                }
            };

            assert_that!("seq def field 36", zero seq_def.zero36, read.prev + 36)?;

            // it doesn't make sense for a sequence to be empty
            assert_that!("seq def pointer", seq_def.pointer != 0, read.prev + 56)?;
            assert_that!("seq def size", seq_def.size > 0, read.prev + 60)?;

            let events = read_events.read(read, seq_def.size, anim_def)?;

            Ok(SeqDef {
                name,
                activation,
                events,
                pointer: seq_def.pointer,
            })
        })
        .collect()
}

pub(crate) fn read_reset_state_pg<R, RE>(
    read: &mut CountingReader<R>,
    anim_def: &AnimDef,
    size: u32,
    pointer: u32,
    mut read_events: RE,
) -> Result<Option<ResetState>>
where
    R: Read,
    RE: ReadEvents<R>,
{
    trace!("Reading reset state");
    let reset_state: SeqDefInfoC = read.read_struct()?;

    assert_that!(
        "reset state name",
        reset_state.name == RESET_SEQUENCE,
        read.prev + 0
    )?;
    assert_that!("reset state flags", reset_state.flags == 0, read.prev + 32)?;
    assert_that!("reset state field 36", zero reset_state.zero36, read.prev + 36)?;
    assert_that!(
        "reset state pointer",
        reset_state.pointer == pointer,
        read.prev + 56
    )?;
    assert_that!("reset state size", reset_state.size == size, read.prev + 60)?;

    if size > 0 {
        assert_that!(
            "reset state pointer",
            reset_state.pointer != 0,
            read.prev + 56
        )?;
        let events = read_events.read(read, size, anim_def)?;
        Ok(Some(ResetState { events, pointer }))
    } else {
        assert_that!(
            "reset state pointer",
            reset_state.pointer == 0,
            read.prev + 56
        )?;
        Ok(None)
    }
}

pub(crate) fn read_reset_state_pm(
    read: &mut CountingReader<impl Read>,
    anim_def: &AnimDef,
) -> Result<ResetState> {
    trace!("Reading reset state");
    let reset_state: SeqDefInfoC = read.read_struct()?;

    assert_that!(
        "reset state name",
        reset_state.name == RESET_SEQUENCE,
        read.prev + 0
    )?;
    assert_that!("reset state flags", reset_state.flags == 0, read.prev + 32)?;
    assert_that!("reset state field 36", zero reset_state.zero36, read.prev + 36)?;

    // it doesn't make sense for a sequence to be empty
    assert_that!(
        "reset state pointer",
        reset_state.pointer != 0,
        read.prev + 56
    )?;
    assert_that!("reset state size", reset_state.size > 0, read.prev + 60)?;

    // this is different than the anim def reset state ptr!
    let pointer = reset_state.pointer;
    let length = reset_state.size;

    let events = mech3ax_anim_events::pm::read_events(read, length, anim_def)?;
    Ok(ResetState { events, pointer })
}
