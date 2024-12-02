use super::{SEQ_ACTIVATION_INITIAL, SEQ_ACTIVATION_ON_CALL};
use crate::common::seq_def::{SeqDefInfoC, RESET_SEQUENCE};
use log::trace;
use mech3ax_anim_events::rc::read_events;
use mech3ax_api_types::anim::{AnimDef, ResetState, SeqActivation, SeqDef, SiScript};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{assert_that, assert_with_msg, Result};
use std::io::Read;

pub(crate) fn read_sequence_defs(
    read: &mut CountingReader<impl Read>,
    anim_def: &AnimDef,
    count: u8,
    scripts: &mut Vec<SiScript>,
) -> Result<Vec<SeqDef>> {
    (0..count)
        .map(|index| {
            trace!("Reading sequence definition {}", index);
            read_sequence_def(read, anim_def, scripts)
        })
        .collect()
}

fn read_sequence_def(
    read: &mut CountingReader<impl Read>,
    anim_def: &AnimDef,
    scripts: &mut Vec<SiScript>,
) -> Result<SeqDef> {
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

    let events = read_events(read, seq_def.size, anim_def, scripts)?;

    Ok(SeqDef {
        name,
        activation,
        events,
        pointer: seq_def.pointer,
    })
}

pub(crate) fn read_reset_state(
    read: &mut CountingReader<impl Read>,
    anim_def: &AnimDef,
    size: u32,
    pointer: u32,
    scripts: &mut Vec<SiScript>,
) -> Result<Option<ResetState>> {
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
        let events = read_events(read, size, anim_def, scripts)?;
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
