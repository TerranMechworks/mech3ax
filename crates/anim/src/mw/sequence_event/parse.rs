use super::*;
use bytemuck::{AnyBitPattern, NoUninit};
use log::trace;
use mech3ax_api_types::anim::events::{
    CallAnimation, CallObjectConnector, CallSequence, Callback, DetonateWeapon, Else, ElseIf,
    EndIf, Event, EventData, EventStart, FogState, FrameBufferEffectColor, If, InvalidateAnimation,
    LightAnimation, LightState, Loop, ObjectActiveState, ObjectAddChild, ObjectConnector,
    ObjectCycleTexture, ObjectMotion, ObjectMotionFromTo, ObjectMotionSiScript,
    ObjectOpacityFromTo, ObjectOpacityState, ObjectRotateState, ObjectScaleState,
    ObjectTranslateState, PufferState, ResetAnimation, Sound, SoundNode, StartOffset,
    StopAnimation, StopSequence,
};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, assert_with_msg, Result};
use mech3ax_types::{impl_as_bytes, u32_to_usize, AsBytes as _};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct EventHeaderC {
    event_type: u8,
    start_offset: u8,
    pad: u16,
    size: u32,
    start_time: f32,
}
impl_as_bytes!(EventHeaderC, 12);

pub fn read_events(
    read: &mut CountingReader<impl Read>,
    length: u32,
    anim_def: &AnimDef,
) -> Result<Vec<Event>> {
    let end_offset = read.offset + u32_to_usize(length);
    let mut events = Vec::new();
    while read.offset < end_offset {
        let header: EventHeaderC = read.read_struct()?;
        trace!(
            "Read anim def event {} at {}",
            header.event_type,
            read.offset
        );
        assert_that!("event header field 02", header.pad == 0, read.prev + 2)?;
        let start_offset = assert_that!("event start offset", enum StartOffset => header.start_offset, read.prev + 1)?;

        let start = if start_offset == StartOffset::Animation && header.start_time == 0.0 {
            None
        } else {
            Some(EventStart {
                offset: start_offset,
                time: header.start_time,
            })
        };

        let actual_size = header.size - EventHeaderC::SIZE;
        assert_that!("event header actual size", actual_size >= 0, read.prev + 4)?;

        let data = match header.event_type {
            Sound::INDEX => EventData::Sound(Sound::read(read, anim_def, actual_size)?),
            SoundNode::INDEX => EventData::SoundNode(SoundNode::read(read, anim_def, actual_size)?),
            LightState::INDEX => {
                EventData::LightState(LightState::read(read, anim_def, actual_size)?)
            }
            LightAnimation::INDEX => {
                EventData::LightAnimation(LightAnimation::read(read, anim_def, actual_size)?)
            }
            ObjectActiveState::INDEX => {
                EventData::ObjectActiveState(ObjectActiveState::read(read, anim_def, actual_size)?)
            }
            ObjectTranslateState::INDEX => EventData::ObjectTranslateState(
                ObjectTranslateState::read(read, anim_def, actual_size)?,
            ),
            ObjectScaleState::INDEX => {
                EventData::ObjectScaleState(ObjectScaleState::read(read, anim_def, actual_size)?)
            }
            ObjectRotateState::INDEX => {
                EventData::ObjectRotateState(ObjectRotateState::read(read, anim_def, actual_size)?)
            }
            ObjectMotion::INDEX => {
                EventData::ObjectMotion(ObjectMotion::read(read, anim_def, actual_size)?)
            }
            ObjectMotionFromTo::INDEX => EventData::ObjectMotionFromTo(ObjectMotionFromTo::read(
                read,
                anim_def,
                actual_size,
            )?),
            ObjectOpacityState::INDEX => EventData::ObjectOpacityState(ObjectOpacityState::read(
                read,
                anim_def,
                actual_size,
            )?),
            ObjectOpacityFromTo::INDEX => EventData::ObjectOpacityFromTo(
                ObjectOpacityFromTo::read(read, anim_def, actual_size)?,
            ),
            ObjectAddChild::INDEX => {
                EventData::ObjectAddChild(ObjectAddChild::read(read, anim_def, actual_size)?)
            }
            ObjectCycleTexture::INDEX => EventData::ObjectCycleTexture(ObjectCycleTexture::read(
                read,
                anim_def,
                actual_size,
            )?),
            ObjectConnector::INDEX => {
                EventData::ObjectConnector(ObjectConnector::read(read, anim_def, actual_size)?)
            }
            CallObjectConnector::INDEX => EventData::CallObjectConnector(
                CallObjectConnector::read(read, anim_def, actual_size)?,
            ),
            CallSequence::INDEX => {
                EventData::CallSequence(CallSequence::read(read, anim_def, actual_size)?)
            }
            StopSequence::INDEX => {
                EventData::StopSequence(StopSequence::read(read, anim_def, actual_size)?)
            }
            CallAnimation::INDEX => {
                EventData::CallAnimation(CallAnimation::read(read, anim_def, actual_size)?)
            }
            StopAnimation::INDEX => {
                EventData::StopAnimation(StopAnimation::read(read, anim_def, actual_size)?)
            }
            ResetAnimation::INDEX => {
                EventData::ResetAnimation(ResetAnimation::read(read, anim_def, actual_size)?)
            }
            InvalidateAnimation::INDEX => EventData::InvalidateAnimation(
                InvalidateAnimation::read(read, anim_def, actual_size)?,
            ),
            FogState::INDEX => EventData::FogState(FogState::read(read, anim_def, actual_size)?),
            Loop::INDEX => EventData::Loop(Loop::read(read, anim_def, actual_size)?),
            If::INDEX => EventData::If(If::read(read, anim_def, actual_size)?),
            Else::INDEX => EventData::Else(Else::read(read, anim_def, actual_size)?),
            ElseIf::INDEX => EventData::Elif(ElseIf::read(read, anim_def, actual_size)?),
            EndIf::INDEX => EventData::Endif(EndIf::read(read, anim_def, actual_size)?),
            Callback::INDEX => EventData::Callback(Callback::read(read, anim_def, actual_size)?),
            FrameBufferEffectColor::INDEX => EventData::FrameBufferEffectColorFromTo(
                FrameBufferEffectColor::read(read, anim_def, actual_size)?,
            ),
            DetonateWeapon::INDEX => {
                EventData::DetonateWeapon(DetonateWeapon::read(read, anim_def, actual_size)?)
            }
            PufferState::INDEX => {
                EventData::PufferState(PufferState::read(read, anim_def, actual_size)?)
            }
            ObjectMotionSiScript::INDEX => EventData::ObjectMotionSIScript(
                ObjectMotionSiScript::read(read, anim_def, actual_size)?,
            ),
            _ => {
                return Err(assert_with_msg!(
                    "Expected valid event type, but was {} (at {})",
                    header.event_type,
                    read.prev + 0
                ));
            }
        };

        events.push(Event { data, start })
    }

    assert_that!("sequence event end", read.offset == end_offset, read.offset)?;
    Ok(events)
}

pub fn write_events(
    write: &mut CountingWriter<impl Write>,
    anim_def: &AnimDef,
    events: &[Event],
) -> Result<()> {
    for event in events {
        let event_type = event_type(event);
        let (start_offset, start_time) = match event.start.as_ref() {
            None => (StartOffset::Animation as u8, 0.0),
            Some(event_start) => (event_start.offset as u8, event_start.time),
        };
        let size = size_event(event);
        write.write_struct(&EventHeaderC {
            event_type,
            start_offset,
            pad: 0,
            size,
            start_time,
        })?;
        write_event(write, anim_def, event)?;
    }
    Ok(())
}

fn event_type(event: &Event) -> u8 {
    match &event.data {
        EventData::Sound(_) => Sound::INDEX,
        EventData::SoundNode(_) => SoundNode::INDEX,
        EventData::LightState(_) => LightState::INDEX,
        EventData::LightAnimation(_) => LightAnimation::INDEX,
        EventData::ObjectActiveState(_) => ObjectActiveState::INDEX,
        EventData::ObjectTranslateState(_) => ObjectTranslateState::INDEX,
        EventData::ObjectScaleState(_) => ObjectScaleState::INDEX,
        EventData::ObjectRotateState(_) => ObjectRotateState::INDEX,
        EventData::ObjectMotion(_) => ObjectMotion::INDEX,
        EventData::ObjectMotionFromTo(_) => ObjectMotionFromTo::INDEX,
        EventData::ObjectOpacityState(_) => ObjectOpacityState::INDEX,
        EventData::ObjectOpacityFromTo(_) => ObjectOpacityFromTo::INDEX,
        EventData::ObjectAddChild(_) => ObjectAddChild::INDEX,
        EventData::ObjectCycleTexture(_) => ObjectCycleTexture::INDEX,
        EventData::ObjectConnector(_) => ObjectConnector::INDEX,
        EventData::CallObjectConnector(_) => CallObjectConnector::INDEX,
        EventData::CallSequence(_) => CallSequence::INDEX,
        EventData::StopSequence(_) => StopSequence::INDEX,
        EventData::CallAnimation(_) => CallAnimation::INDEX,
        EventData::StopAnimation(_) => StopAnimation::INDEX,
        EventData::ResetAnimation(_) => ResetAnimation::INDEX,
        EventData::InvalidateAnimation(_) => InvalidateAnimation::INDEX,
        EventData::FogState(_) => FogState::INDEX,
        EventData::Loop(_) => Loop::INDEX,
        EventData::If(_) => If::INDEX,
        EventData::Else(_) => Else::INDEX,
        EventData::Elif(_) => ElseIf::INDEX,
        EventData::Endif(_) => EndIf::INDEX,
        EventData::Callback(_) => Callback::INDEX,
        EventData::FrameBufferEffectColorFromTo(_) => FrameBufferEffectColor::INDEX,
        EventData::DetonateWeapon(_) => DetonateWeapon::INDEX,
        EventData::PufferState(_) => PufferState::INDEX,
        EventData::ObjectMotionSIScript(_) => ObjectMotionSiScript::INDEX,
    }
}

fn size_event(event: &Event) -> u32 {
    let size = match &event.data {
        EventData::Sound(_) => Sound::SIZE,
        EventData::SoundNode(_) => SoundNode::SIZE,
        EventData::LightState(_) => LightState::SIZE,
        EventData::LightAnimation(_) => LightAnimation::SIZE,
        EventData::ObjectActiveState(_) => ObjectActiveState::SIZE,
        EventData::ObjectTranslateState(_) => ObjectTranslateState::SIZE,
        EventData::ObjectScaleState(_) => ObjectScaleState::SIZE,
        EventData::ObjectRotateState(_) => ObjectRotateState::SIZE,
        EventData::ObjectMotion(_) => ObjectMotion::SIZE,
        EventData::ObjectMotionFromTo(_) => ObjectMotionFromTo::SIZE,
        EventData::ObjectOpacityState(_) => ObjectOpacityState::SIZE,
        EventData::ObjectOpacityFromTo(_) => ObjectOpacityFromTo::SIZE,
        EventData::ObjectAddChild(_) => ObjectAddChild::SIZE,
        EventData::ObjectCycleTexture(_) => ObjectCycleTexture::SIZE,
        EventData::ObjectConnector(_) => ObjectConnector::SIZE,
        EventData::CallObjectConnector(_) => CallObjectConnector::SIZE,
        EventData::CallSequence(_) => CallSequence::SIZE,
        EventData::StopSequence(_) => StopSequence::SIZE,
        EventData::CallAnimation(_) => CallAnimation::SIZE,
        EventData::StopAnimation(_) => StopAnimation::SIZE,
        EventData::ResetAnimation(_) => ResetAnimation::SIZE,
        EventData::InvalidateAnimation(_) => InvalidateAnimation::SIZE,
        EventData::FogState(_) => FogState::SIZE,
        EventData::Loop(_) => Loop::SIZE,
        EventData::If(_) => If::SIZE,
        EventData::Else(_) => Else::SIZE,
        EventData::Elif(_) => ElseIf::SIZE,
        EventData::Endif(_) => EndIf::SIZE,
        EventData::Callback(_) => Callback::SIZE,
        EventData::FrameBufferEffectColorFromTo(_) => FrameBufferEffectColor::SIZE,
        EventData::DetonateWeapon(_) => DetonateWeapon::SIZE,
        EventData::PufferState(_) => PufferState::SIZE,
        EventData::ObjectMotionSIScript(script) => object_motion_si_script_size(script),
    };
    size + EventHeaderC::SIZE
}

fn write_event(
    write: &mut CountingWriter<impl Write>,
    anim_def: &AnimDef,
    event: &Event,
) -> Result<()> {
    match &event.data {
        EventData::Sound(data) => data.write(write, anim_def),
        EventData::SoundNode(data) => data.write(write, anim_def),
        EventData::LightState(data) => data.write(write, anim_def),
        EventData::LightAnimation(data) => data.write(write, anim_def),
        EventData::ObjectActiveState(data) => data.write(write, anim_def),
        EventData::ObjectTranslateState(data) => data.write(write, anim_def),
        EventData::ObjectScaleState(data) => data.write(write, anim_def),
        EventData::ObjectRotateState(data) => data.write(write, anim_def),
        EventData::ObjectMotion(data) => data.write(write, anim_def),
        EventData::ObjectMotionFromTo(data) => data.write(write, anim_def),
        EventData::ObjectOpacityState(data) => data.write(write, anim_def),
        EventData::ObjectOpacityFromTo(data) => data.write(write, anim_def),
        EventData::ObjectAddChild(data) => data.write(write, anim_def),
        EventData::ObjectCycleTexture(data) => data.write(write, anim_def),
        EventData::ObjectConnector(data) => data.write(write, anim_def),
        EventData::CallObjectConnector(data) => data.write(write, anim_def),
        EventData::CallSequence(data) => data.write(write, anim_def),
        EventData::StopSequence(data) => data.write(write, anim_def),
        EventData::CallAnimation(data) => data.write(write, anim_def),
        EventData::StopAnimation(data) => data.write(write, anim_def),
        EventData::ResetAnimation(data) => data.write(write, anim_def),
        EventData::InvalidateAnimation(data) => data.write(write, anim_def),
        EventData::FogState(data) => data.write(write, anim_def),
        EventData::Loop(data) => data.write(write, anim_def),
        EventData::If(data) => data.write(write, anim_def),
        EventData::Else(data) => data.write(write, anim_def),
        EventData::Elif(data) => data.write(write, anim_def),
        EventData::Endif(data) => data.write(write, anim_def),
        EventData::Callback(data) => data.write(write, anim_def),
        EventData::FrameBufferEffectColorFromTo(data) => data.write(write, anim_def),
        EventData::DetonateWeapon(data) => data.write(write, anim_def),
        EventData::PufferState(data) => data.write(write, anim_def),
        EventData::ObjectMotionSIScript(data) => data.write(write, anim_def),
    }
}

pub fn size_events(events: &[Event]) -> u32 {
    let mut size = 0;
    for event in events {
        size += size_event(event);
    }
    size
}
