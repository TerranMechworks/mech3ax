use super::SeqDefInfoC;
use log::trace;
use mech3ax_api_types::anim::events::Event;
use mech3ax_api_types::anim::{AnimDef, ResetState, SiScript};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{assert_with_msg, Result};
use mech3ax_types::Ascii;
use std::io::Write;

pub(crate) trait WriteEvents<W: Write> {
    fn size<'a>(&'a self, events: &'a [Event]) -> Option<u32>;

    fn write<'a>(
        &'a self,
        write: &'a mut CountingWriter<W>,
        anim_def: &'a AnimDef,
        events: &'a [Event],
    ) -> Result<()>;
}

#[derive(Debug)]
pub(crate) struct WriteEventsMw<'a> {
    pub(crate) scripts: &'a [SiScript],
}

impl<W: Write> WriteEvents<W> for WriteEventsMw<'_> {
    fn size(&self, events: &[Event]) -> Option<u32> {
        mech3ax_anim_events::mw::size_events(events, self.scripts)
    }

    fn write(
        &self,
        write: &mut CountingWriter<W>,
        anim_def: &AnimDef,
        events: &[Event],
    ) -> Result<()> {
        mech3ax_anim_events::mw::write_events(write, anim_def, events, self.scripts)
    }
}

#[derive(Debug)]
pub(crate) struct WriteEventsPm;

impl<W: Write> WriteEvents<W> for WriteEventsPm {
    fn size(&self, events: &[Event]) -> Option<u32> {
        mech3ax_anim_events::pm::size_events(events)
    }

    fn write(
        &self,
        write: &mut CountingWriter<W>,
        anim_def: &AnimDef,
        events: &[Event],
    ) -> Result<()> {
        mech3ax_anim_events::pm::write_events(write, anim_def, events)
    }
}

#[derive(Debug)]
pub(crate) struct WriteEventsRc<'a> {
    pub(crate) scripts: &'a [SiScript],
}

impl<W: Write> WriteEvents<W> for WriteEventsRc<'_> {
    fn size(&self, events: &[Event]) -> Option<u32> {
        mech3ax_anim_events::rc::size_events(events, self.scripts)
    }

    fn write(
        &self,
        write: &mut CountingWriter<W>,
        anim_def: &AnimDef,
        events: &[Event],
    ) -> Result<()> {
        mech3ax_anim_events::rc::write_events(write, anim_def, events, self.scripts)
    }
}

pub(crate) fn write_sequence_defs<W, WE>(
    write: &mut CountingWriter<W>,
    anim_def: &AnimDef,
    write_events: WE,
) -> Result<()>
where
    W: Write,
    WE: WriteEvents<W>,
{
    for (index, seq_def) in anim_def.sequences.iter().enumerate() {
        trace!("Writing sequence definition {}", index);

        let name = Ascii::from_str_padded(&seq_def.name);
        let size = write_events
            .size(&seq_def.events)
            .ok_or_else(|| assert_with_msg!("Sequence definition {} event data overflow", index))?;

        let seq_def_c = SeqDefInfoC {
            name,
            seq_state: seq_def.seq_state.maybe(),
            reset_state: seq_def.reset_state.maybe(),
            pad34: 0,
            loop_time: 0.0,
            event_time: 0.0,
            seq_time: 0.0,
            loop_count: 0,
            curr_event_ptr: 0,
            pointer: seq_def.pointer,
            size,
        };
        write.write_struct(&seq_def_c)?;

        write_events.write(write, anim_def, &seq_def.events)?;
    }
    Ok(())
}

pub(crate) fn write_reset_state_pg<W, WE>(
    write: &mut CountingWriter<W>,
    anim_def: &AnimDef,
    size: u32,
    write_events: WE,
) -> Result<()>
where
    W: Write,
    WE: WriteEvents<W>,
{
    trace!("Writing reset state");

    let pointer = anim_def
        .reset_state
        .as_ref()
        .map(|state| state.pointer)
        .unwrap_or(0);

    let seq_def = SeqDefInfoC::make_reset_state(pointer, size);
    write.write_struct(&seq_def)?;

    if let Some(state) = &anim_def.reset_state {
        write_events.write(write, anim_def, &state.events)?;
    }
    Ok(())
}

pub(crate) fn write_reset_state_pm(
    write: &mut CountingWriter<impl Write>,
    anim_def: &AnimDef,
    reset_state: &ResetState,
) -> Result<()> {
    trace!("Writing reset state");

    let size = mech3ax_anim_events::pm::size_events(&reset_state.events)
        .ok_or_else(|| assert_with_msg!("Reset state event data overflow"))?;

    let seq_def = SeqDefInfoC::make_reset_state(reset_state.pointer, size);
    write.write_struct(&seq_def)?;

    mech3ax_anim_events::pm::write_events(write, anim_def, &reset_state.events)?;
    Ok(())
}
