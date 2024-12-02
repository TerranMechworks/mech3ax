use super::{SEQ_ACTIVATION_INITIAL, SEQ_ACTIVATION_ON_CALL};
use crate::common::seq_def::{SeqDefInfoC, RESET_SEQUENCE};
use log::trace;
use mech3ax_anim_events::mw::{size_events, write_events};
use mech3ax_api_types::anim::{AnimDef, SeqActivation, SiScript};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{assert_with_msg, Result};
use mech3ax_types::{Ascii, Zeros};
use std::io::Write;

pub(crate) fn write_sequence_defs(
    write: &mut CountingWriter<impl Write>,
    anim_def: &AnimDef,
    scripts: &[SiScript],
) -> Result<()> {
    for (index, seq_def) in anim_def.sequences.iter().enumerate() {
        trace!("Writing sequence definition {}", index);

        let name = Ascii::from_str_padded(&seq_def.name);
        let flags = match seq_def.activation {
            SeqActivation::OnCall => SEQ_ACTIVATION_ON_CALL,
            SeqActivation::Initial => SEQ_ACTIVATION_INITIAL,
        };
        let size = size_events(&seq_def.events, scripts)
            .ok_or_else(|| assert_with_msg!("Sequence definition {} event data overflow", index))?;

        let seq_def_c = SeqDefInfoC {
            name,
            flags,
            zero36: Zeros::new(),
            pointer: seq_def.pointer,
            size,
        };
        write.write_struct(&seq_def_c)?;

        write_events(write, anim_def, &seq_def.events, scripts)?;
    }
    Ok(())
}

pub(crate) fn write_reset_state(
    write: &mut CountingWriter<impl Write>,
    anim_def: &AnimDef,
    size: u32,
    scripts: &[SiScript],
) -> Result<()> {
    trace!("Writing reset state");

    let pointer = anim_def
        .reset_state
        .as_ref()
        .map(|state| state.pointer)
        .unwrap_or(0);

    let seq_def = SeqDefInfoC {
        name: RESET_SEQUENCE,
        flags: 0,
        zero36: Zeros::new(),
        pointer,
        size,
    };
    write.write_struct(&seq_def)?;

    if let Some(state) = &anim_def.reset_state {
        write_events(write, anim_def, &state.events, scripts)?;
    }
    Ok(())
}
