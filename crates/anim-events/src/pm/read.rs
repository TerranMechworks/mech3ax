use crate::events::EventPm;
use crate::{EventHeaderC, EventType};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::anim::events::*;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{Result, assert_that, assert_with_msg};
use mech3ax_types::{AsBytes as _, u32_to_usize};
use std::io::Read;

macro_rules! read {
    ($evt:ty, $read:ident, $anim_def:ident, $data_size:ident) => {{
        log::trace!(
            "Reading event data `{}` of {} bytes (at {})",
            stringify!($evt),
            $data_size,
            $read.offset,
        );
        <$evt as EventPm>::read($read, $anim_def, $data_size)?
    }};
}

pub fn read_events(
    read: &mut CountingReader<impl Read>,
    length: u32,
    anim_def: &AnimDef,
) -> Result<Vec<Event>> {
    let end_offset = read.offset + u32_to_usize(length);
    let mut events = Vec::new();
    while read.offset < end_offset {
        log::trace!("Reading seq event {}", events.len());
        let header: EventHeaderC = read.read_struct()?;

        let event_type = assert_that!("event header type", enum header.event_type, read.prev + 0)?;
        let start_offset =
            assert_that!("event start offset", enum header.start_offset, read.prev + 1)?;
        assert_that!("event header field 02", header.pad == 0, read.prev + 2)?;

        let data_size = header.size.checked_sub(EventHeaderC::SIZE).ok_or_else(|| {
            assert_with_msg!(
                "Expected event size > {}, but was {} (at {})",
                EventHeaderC::SIZE,
                header.size,
                read.prev + 4,
            )
        })?;

        let start = if start_offset == StartOffset::Animation && header.start_time == 0.0 {
            None
        } else {
            Some(EventStart {
                offset: start_offset,
                time: header.start_time,
            })
        };

        let data = match event_type {
            EventType::Sound => EventData::Sound(read!(Sound, read, anim_def, data_size)),
            EventType::SoundNode => {
                EventData::SoundNode(read!(SoundNode, read, anim_def, data_size))
            }
            EventType::Effect => EventData::Effect(read!(Effect, read, anim_def, data_size)),
            EventType::LightState => {
                EventData::LightState(read!(LightState, read, anim_def, data_size))
            }
            EventType::LightAnimation => {
                EventData::LightAnimation(read!(LightAnimation, read, anim_def, data_size))
            }
            EventType::ObjectActiveState => {
                EventData::ObjectActiveState(read!(ObjectActiveState, read, anim_def, data_size))
            }
            EventType::ObjectTranslateState => EventData::ObjectTranslateState(read!(
                ObjectTranslateState,
                read,
                anim_def,
                data_size
            )),
            EventType::ObjectScaleState => {
                EventData::ObjectScaleState(read!(ObjectScaleState, read, anim_def, data_size))
            }
            EventType::ObjectRotateState => {
                EventData::ObjectRotateState(read!(ObjectRotateState, read, anim_def, data_size))
            }
            EventType::ObjectMotion => {
                EventData::ObjectMotion(read!(ObjectMotion, read, anim_def, data_size))
            }
            EventType::ObjectMotionFromTo => {
                EventData::ObjectMotionFromTo(read!(ObjectMotionFromTo, read, anim_def, data_size))
            }
            EventType::ObjectMotionSiScript => EventData::ObjectMotionSiScript(read!(
                ObjectMotionSiScript,
                read,
                anim_def,
                data_size
            )),
            EventType::ObjectOpacityState => {
                EventData::ObjectOpacityState(read!(ObjectOpacityState, read, anim_def, data_size))
            }
            EventType::ObjectOpacityFromTo => EventData::ObjectOpacityFromTo(read!(
                ObjectOpacityFromTo,
                read,
                anim_def,
                data_size
            )),
            EventType::ObjectAddChild => {
                EventData::ObjectAddChild(read!(ObjectAddChild, read, anim_def, data_size))
            }
            EventType::ObjectDeleteChild => {
                EventData::ObjectDeleteChild(read!(ObjectDeleteChild, read, anim_def, data_size))
            }
            EventType::ObjectCycleTexture => {
                EventData::ObjectCycleTexture(read!(ObjectCycleTexture, read, anim_def, data_size))
            }
            EventType::ObjectConnector => {
                EventData::ObjectConnector(read!(ObjectConnector, read, anim_def, data_size))
            }
            EventType::CallObjectConnector => EventData::CallObjectConnector(read!(
                CallObjectConnector,
                read,
                anim_def,
                data_size
            )),
            EventType::CameraState => {
                EventData::CameraState(read!(CameraState, read, anim_def, data_size))
            }
            EventType::CameraFromTo => {
                EventData::CameraFromTo(read!(CameraFromTo, read, anim_def, data_size))
            }
            EventType::CallSequence => {
                EventData::CallSequence(read!(CallSequence, read, anim_def, data_size))
            }
            EventType::StopSequence => {
                EventData::StopSequence(read!(StopSequence, read, anim_def, data_size))
            }
            EventType::CallAnimation => {
                EventData::CallAnimation(read!(CallAnimation, read, anim_def, data_size))
            }
            EventType::StopAnimation => {
                EventData::StopAnimation(read!(StopAnimation, read, anim_def, data_size))
            }
            EventType::ResetAnimation => {
                EventData::ResetAnimation(read!(ResetAnimation, read, anim_def, data_size))
            }
            EventType::InvalidateAnimation => EventData::InvalidateAnimation(read!(
                InvalidateAnimation,
                read,
                anim_def,
                data_size
            )),
            EventType::FogState => EventData::FogState(read!(FogState, read, anim_def, data_size)),
            EventType::Loop => EventData::Loop(read!(Loop, read, anim_def, data_size)),
            EventType::If => EventData::If(read!(If, read, anim_def, data_size)),
            EventType::Else => EventData::Else(read!(Else, read, anim_def, data_size)),
            EventType::Elseif => EventData::Elseif(read!(Elseif, read, anim_def, data_size)),
            EventType::Endif => EventData::Endif(read!(Endif, read, anim_def, data_size)),
            EventType::Callback => EventData::Callback(read!(Callback, read, anim_def, data_size)),
            EventType::FbfxColorFromTo => {
                EventData::FbfxColorFromTo(read!(FbfxColorFromTo, read, anim_def, data_size))
            }
            EventType::FbfxCsinwaveFromTo => {
                EventData::FbfxCsinwaveFromTo(read!(FbfxCsinwaveFromTo, read, anim_def, data_size))
            }
            EventType::AnimVerbose => {
                EventData::AnimVerbose(read!(AnimVerbose, read, anim_def, data_size))
            }
            EventType::DetonateWeapon => {
                EventData::DetonateWeapon(read!(DetonateWeapon, read, anim_def, data_size))
            }
            EventType::PufferState => {
                EventData::PufferState(read!(PufferState, read, anim_def, data_size))
            }
        };

        events.push(Event { start, data });
    }

    assert_that!("sequence event end", read.offset == end_offset, read.offset)?;
    Ok(events)
}
