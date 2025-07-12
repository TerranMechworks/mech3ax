use crate::events::{EventMw, object_motion_si_script};
use crate::{EventHeaderC, event_type};
use mech3ax_api_types::anim::events::*;
use mech3ax_api_types::anim::{AnimDef, SiScript};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{Result, assert_with_msg};
use mech3ax_types::AsBytes as _;
use std::io::Write;

macro_rules! write {
    ($evt:ty, $write:ident, $anim_def:ident, $data:ident) => {{
        let size = <$evt as EventMw>::size($data);
        log::trace!(
            "Writing event data `{}` of {} bytes (at {})",
            stringify!($evt),
            size.unwrap_or(0),
            $write.offset,
        );
        <$evt as EventMw>::write($data, $write, $anim_def)
    }};
}

pub fn write_events(
    write: &mut CountingWriter<impl Write>,
    anim_def: &AnimDef,
    events: &[Event],
    scripts: &[SiScript],
) -> Result<()> {
    for (index, event) in events.iter().enumerate() {
        log::trace!("Writing seq event {}", index);

        let event_type = event_type(&event.data);

        let (start_offset, start_time) = match event.start.as_ref() {
            None => (StartOffset::Animation.maybe(), 0.0),
            Some(event_start) => (event_start.offset.maybe(), event_start.time),
        };
        // this is unlikely to overflow, as the sequence definition should have
        // been sized by this point.
        let size = size_event(event, scripts)
            .and_then(|size| size.checked_add(EventHeaderC::SIZE))
            .ok_or_else(|| assert_with_msg!("Seq event {} size overflow", index))?;

        let header = EventHeaderC {
            event_type,
            start_offset,
            pad: 0,
            size,
            start_time,
        };
        write.write_struct(&header)?;

        write_event(write, anim_def, event, scripts)?;
    }
    Ok(())
}

fn write_event(
    write: &mut CountingWriter<impl Write>,
    anim_def: &AnimDef,
    event: &Event,
    scripts: &[SiScript],
) -> Result<()> {
    match &event.data {
        EventData::Sound(data) => write!(Sound, write, anim_def, data),
        EventData::SoundNode(data) => write!(SoundNode, write, anim_def, data),
        EventData::Effect(data) => write!(Effect, write, anim_def, data),
        EventData::LightState(data) => write!(LightState, write, anim_def, data),
        EventData::LightAnimation(data) => write!(LightAnimation, write, anim_def, data),
        EventData::ObjectActiveState(data) => write!(ObjectActiveState, write, anim_def, data),
        EventData::ObjectTranslateState(data) => {
            write!(ObjectTranslateState, write, anim_def, data)
        }
        EventData::ObjectScaleState(data) => write!(ObjectScaleState, write, anim_def, data),
        EventData::ObjectRotateState(data) => write!(ObjectRotateState, write, anim_def, data),
        EventData::ObjectMotion(data) => write!(ObjectMotion, write, anim_def, data),
        EventData::ObjectMotionFromTo(data) => write!(ObjectMotionFromTo, write, anim_def, data),
        EventData::ObjectMotionSiScript(data) => {
            let size = object_motion_si_script::size_mw(data, scripts);
            log::trace!(
                "Writing event data `ObjectMotionSiScript` of {} bytes (at {})",
                size.unwrap_or(0),
                write.offset,
            );
            object_motion_si_script::write_mw(write, anim_def, data, scripts)
        }
        EventData::ObjectOpacityState(data) => write!(ObjectOpacityState, write, anim_def, data),
        EventData::ObjectOpacityFromTo(data) => write!(ObjectOpacityFromTo, write, anim_def, data),
        EventData::ObjectAddChild(data) => write!(ObjectAddChild, write, anim_def, data),
        EventData::ObjectDeleteChild(data) => write!(ObjectDeleteChild, write, anim_def, data),
        EventData::ObjectCycleTexture(data) => write!(ObjectCycleTexture, write, anim_def, data),
        EventData::ObjectConnector(data) => write!(ObjectConnector, write, anim_def, data),
        EventData::CallObjectConnector(data) => write!(CallObjectConnector, write, anim_def, data),
        EventData::CameraState(data) => write!(CameraState, write, anim_def, data),
        EventData::CameraFromTo(data) => write!(CameraFromTo, write, anim_def, data),
        EventData::CallSequence(data) => write!(CallSequence, write, anim_def, data),
        EventData::StopSequence(data) => write!(StopSequence, write, anim_def, data),
        EventData::CallAnimation(data) => write!(CallAnimation, write, anim_def, data),
        EventData::StopAnimation(data) => write!(StopAnimation, write, anim_def, data),
        EventData::ResetAnimation(data) => write!(ResetAnimation, write, anim_def, data),
        EventData::InvalidateAnimation(data) => write!(InvalidateAnimation, write, anim_def, data),
        EventData::FogState(data) => write!(FogState, write, anim_def, data),
        EventData::Loop(data) => write!(Loop, write, anim_def, data),
        EventData::If(data) => write!(If, write, anim_def, data),
        EventData::Else(data) => write!(Else, write, anim_def, data),
        EventData::Elseif(data) => write!(Elseif, write, anim_def, data),
        EventData::Endif(data) => write!(Endif, write, anim_def, data),
        EventData::Callback(data) => write!(Callback, write, anim_def, data),
        EventData::FbfxColorFromTo(data) => {
            write!(FbfxColorFromTo, write, anim_def, data)
        }
        EventData::FbfxCsinwaveFromTo(data) => {
            write!(FbfxCsinwaveFromTo, write, anim_def, data)
        }
        EventData::AnimVerbose(data) => {
            write!(AnimVerbose, write, anim_def, data)
        }
        EventData::DetonateWeapon(data) => write!(DetonateWeapon, write, anim_def, data),
        EventData::PufferState(data) => write!(PufferState, write, anim_def, data),
    }
}

fn size_event(event: &Event, scripts: &[SiScript]) -> Option<u32> {
    match &event.data {
        EventData::Sound(inner) => inner.size(),
        EventData::SoundNode(inner) => inner.size(),
        EventData::Effect(inner) => inner.size(),
        EventData::LightState(inner) => inner.size(),
        EventData::LightAnimation(inner) => inner.size(),
        EventData::ObjectActiveState(inner) => inner.size(),
        EventData::ObjectTranslateState(inner) => inner.size(),
        EventData::ObjectScaleState(inner) => inner.size(),
        EventData::ObjectRotateState(inner) => inner.size(),
        EventData::ObjectMotion(inner) => inner.size(),
        EventData::ObjectMotionFromTo(inner) => inner.size(),
        EventData::ObjectMotionSiScript(data) => object_motion_si_script::size_mw(data, scripts),
        EventData::ObjectOpacityState(inner) => inner.size(),
        EventData::ObjectOpacityFromTo(inner) => inner.size(),
        EventData::ObjectAddChild(inner) => inner.size(),
        EventData::ObjectDeleteChild(inner) => inner.size(),
        EventData::ObjectCycleTexture(inner) => inner.size(),
        EventData::ObjectConnector(inner) => inner.size(),
        EventData::CallObjectConnector(inner) => inner.size(),
        EventData::CameraState(inner) => inner.size(),
        EventData::CameraFromTo(inner) => inner.size(),
        EventData::CallSequence(inner) => inner.size(),
        EventData::StopSequence(inner) => inner.size(),
        EventData::CallAnimation(inner) => inner.size(),
        EventData::StopAnimation(inner) => inner.size(),
        EventData::ResetAnimation(inner) => inner.size(),
        EventData::InvalidateAnimation(inner) => inner.size(),
        EventData::FogState(inner) => inner.size(),
        EventData::Loop(inner) => inner.size(),
        EventData::If(inner) => inner.size(),
        EventData::Else(inner) => inner.size(),
        EventData::Elseif(inner) => inner.size(),
        EventData::Endif(inner) => inner.size(),
        EventData::Callback(inner) => inner.size(),
        EventData::FbfxColorFromTo(inner) => inner.size(),
        EventData::FbfxCsinwaveFromTo(inner) => inner.size(),
        EventData::AnimVerbose(inner) => inner.size(),
        EventData::DetonateWeapon(inner) => inner.size(),
        EventData::PufferState(inner) => inner.size(),
    }
}

pub fn size_events(events: &[Event], scripts: &[SiScript]) -> Option<u32> {
    let mut size = 0u32;
    for event in events {
        size = size.checked_add(EventHeaderC::SIZE)?;
        let event_size = size_event(event, scripts)?;
        size = size.checked_add(event_size)?;
    }
    Some(size)
}
