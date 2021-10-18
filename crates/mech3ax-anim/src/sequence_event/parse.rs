use super::*;
use crate::AnimDef;
use log::trace;
use mech3ax_common::assert::AssertionError;
use mech3ax_common::io_ext::{CountingReader, WriteHelper};
use mech3ax_common::size::ReprSize;
use mech3ax_common::{assert_that, static_assert_size, Result};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[derive(Debug, Serialize, Deserialize)]
pub enum EventData {
    Sound(sound::Sound),
    SoundNode(sound_node::SoundNode),
    LightState(light_state::LightState),
    LightAnimation(light_animation::LightAnimation),
    ObjectActiveState(object_active_state::ObjectActiveState),
    ObjectTranslateState(object_translate_state::ObjectTranslateState),
    ObjectScaleState(object_scale_state::ObjectScaleState),
    ObjectRotateState(object_rotate_state::ObjectRotateState),
    ObjectMotion(object_motion::ObjectMotion),
    ObjectMotionFromTo(object_motion_from_to::ObjectMotionFromTo),
    ObjectMotionSIScript(object_motion_si_script::ObjectMotionSiScript),
    ObjectOpacityState(object_opacity_state::ObjectOpacityState),
    ObjectOpacityFromTo(object_opacity_from_to::ObjectOpacityFromTo),
    ObjectAddChild(object_add_child::ObjectAddChild),
    ObjectCycleTexture(object_cycle_texture::ObjectCycleTexture),
    ObjectConnector(object_connector::ObjectConnector),
    CallObjectConnector(call_object_connector::CallObjectConnector),
    CallSequence(sequence::CallSequence),
    StopSequence(sequence::StopSequence),
    CallAnimation(call_animation::CallAnimation),
    StopAnimation(animation::StopAnimation),
    ResetAnimation(animation::ResetAnimation),
    InvalidateAnimation(animation::InvalidateAnimation),
    FogState(fog_state::FogState),
    Loop(control_flow::Loop),
    If(control_flow::If),
    Else(control_flow::Else),
    Elif(control_flow::ElseIf),
    Endif(control_flow::EndIf),
    Callback(control_flow::Callback),
    FrameBufferEffectColorFromTo(fbfx_color_from_to::FrameBufferEffectColor),
    DetonateWeapon(detonate_weapon::DetonateWeapon),
    PufferState(puffer_state::PufferState),
}

#[repr(C)]
struct EventHeaderC {
    event_type: u8,
    start_offset: u8,
    pad: u16,
    size: u32,
    start_time: f32,
}
static_assert_size!(EventHeaderC, 12);

#[derive(Debug, Serialize, Deserialize, FromPrimitive, PartialEq)]
#[repr(u8)]
pub enum StartOffset {
    Animation = 1,
    Sequence = 2,
    Event = 3,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub data: EventData,
    pub start: Option<(StartOffset, f32)>,
}

pub fn read_events<R: Read>(
    read: &mut CountingReader<R>,
    length: u32,
    anim_def: &AnimDef,
) -> Result<Vec<Event>> {
    let end_offset = read.offset + length;
    let mut events = Vec::new();
    while read.offset < end_offset {
        let header: EventHeaderC = read.read_struct()?;
        trace!(
            "Read anim def event {} at {}",
            header.event_type,
            read.offset
        );
        assert_that!("event header field 02", header.pad == 0, read.prev + 2)?;
        let start_offset: StartOffset =
            FromPrimitive::from_u8(header.start_offset).ok_or_else(|| {
                AssertionError(format!(
                    "Expected valid event start offset, but was {} (at {})",
                    header.start_offset,
                    read.prev + 1
                ))
            })?;

        let start = if start_offset == StartOffset::Animation && header.start_time == 0.0 {
            None
        } else {
            Some((start_offset, header.start_time))
        };

        let actual_size = header.size - EventHeaderC::SIZE;
        assert_that!("event header actual size", actual_size >= 0, read.prev + 4)?;

        let data = match header.event_type {
            sound::Sound::INDEX => {
                EventData::Sound(sound::Sound::read(read, anim_def, actual_size)?)
            }
            sound_node::SoundNode::INDEX => {
                EventData::SoundNode(sound_node::SoundNode::read(read, anim_def, actual_size)?)
            }
            light_state::LightState::INDEX => {
                EventData::LightState(light_state::LightState::read(read, anim_def, actual_size)?)
            }
            light_animation::LightAnimation::INDEX => EventData::LightAnimation(
                light_animation::LightAnimation::read(read, anim_def, actual_size)?,
            ),
            object_active_state::ObjectActiveState::INDEX => EventData::ObjectActiveState(
                object_active_state::ObjectActiveState::read(read, anim_def, actual_size)?,
            ),
            object_translate_state::ObjectTranslateState::INDEX => EventData::ObjectTranslateState(
                object_translate_state::ObjectTranslateState::read(read, anim_def, actual_size)?,
            ),
            object_scale_state::ObjectScaleState::INDEX => EventData::ObjectScaleState(
                object_scale_state::ObjectScaleState::read(read, anim_def, actual_size)?,
            ),
            object_rotate_state::ObjectRotateState::INDEX => EventData::ObjectRotateState(
                object_rotate_state::ObjectRotateState::read(read, anim_def, actual_size)?,
            ),
            object_motion::ObjectMotion::INDEX => EventData::ObjectMotion(
                object_motion::ObjectMotion::read(read, anim_def, actual_size)?,
            ),
            object_motion_from_to::ObjectMotionFromTo::INDEX => EventData::ObjectMotionFromTo(
                object_motion_from_to::ObjectMotionFromTo::read(read, anim_def, actual_size)?,
            ),
            object_opacity_state::ObjectOpacityState::INDEX => EventData::ObjectOpacityState(
                object_opacity_state::ObjectOpacityState::read(read, anim_def, actual_size)?,
            ),
            object_opacity_from_to::ObjectOpacityFromTo::INDEX => EventData::ObjectOpacityFromTo(
                object_opacity_from_to::ObjectOpacityFromTo::read(read, anim_def, actual_size)?,
            ),
            object_add_child::ObjectAddChild::INDEX => EventData::ObjectAddChild(
                object_add_child::ObjectAddChild::read(read, anim_def, actual_size)?,
            ),
            object_cycle_texture::ObjectCycleTexture::INDEX => EventData::ObjectCycleTexture(
                object_cycle_texture::ObjectCycleTexture::read(read, anim_def, actual_size)?,
            ),
            object_connector::ObjectConnector::INDEX => EventData::ObjectConnector(
                object_connector::ObjectConnector::read(read, anim_def, actual_size)?,
            ),
            call_object_connector::CallObjectConnector::INDEX => EventData::CallObjectConnector(
                call_object_connector::CallObjectConnector::read(read, anim_def, actual_size)?,
            ),
            sequence::CallSequence::INDEX => {
                EventData::CallSequence(sequence::CallSequence::read(read, anim_def, actual_size)?)
            }
            sequence::StopSequence::INDEX => {
                EventData::StopSequence(sequence::StopSequence::read(read, anim_def, actual_size)?)
            }
            call_animation::CallAnimation::INDEX => EventData::CallAnimation(
                call_animation::CallAnimation::read(read, anim_def, actual_size)?,
            ),
            animation::StopAnimation::INDEX => EventData::StopAnimation(
                animation::StopAnimation::read(read, anim_def, actual_size)?,
            ),
            animation::ResetAnimation::INDEX => EventData::ResetAnimation(
                animation::ResetAnimation::read(read, anim_def, actual_size)?,
            ),
            animation::InvalidateAnimation::INDEX => EventData::InvalidateAnimation(
                animation::InvalidateAnimation::read(read, anim_def, actual_size)?,
            ),
            fog_state::FogState::INDEX => {
                EventData::FogState(fog_state::FogState::read(read, anim_def, actual_size)?)
            }
            control_flow::Loop::INDEX => {
                EventData::Loop(control_flow::Loop::read(read, anim_def, actual_size)?)
            }
            control_flow::If::INDEX => {
                EventData::If(control_flow::If::read(read, anim_def, actual_size)?)
            }
            control_flow::Else::INDEX => {
                EventData::Else(control_flow::Else::read(read, anim_def, actual_size)?)
            }
            control_flow::ElseIf::INDEX => {
                EventData::Elif(control_flow::ElseIf::read(read, anim_def, actual_size)?)
            }
            control_flow::EndIf::INDEX => {
                EventData::Endif(control_flow::EndIf::read(read, anim_def, actual_size)?)
            }
            control_flow::Callback::INDEX => {
                EventData::Callback(control_flow::Callback::read(read, anim_def, actual_size)?)
            }
            fbfx_color_from_to::FrameBufferEffectColor::INDEX => {
                EventData::FrameBufferEffectColorFromTo(
                    fbfx_color_from_to::FrameBufferEffectColor::read(read, anim_def, actual_size)?,
                )
            }
            detonate_weapon::DetonateWeapon::INDEX => EventData::DetonateWeapon(
                detonate_weapon::DetonateWeapon::read(read, anim_def, actual_size)?,
            ),
            puffer_state::PufferState::INDEX => EventData::PufferState(
                puffer_state::PufferState::read(read, anim_def, actual_size)?,
            ),
            object_motion_si_script::ObjectMotionSiScript::INDEX => {
                EventData::ObjectMotionSIScript(
                    object_motion_si_script::ObjectMotionSiScript::read(
                        read,
                        anim_def,
                        actual_size,
                    )?,
                )
            }
            _ => {
                let msg = format!(
                    "Expected valid event type, but was {} (at {})",
                    header.event_type,
                    read.prev + 0
                );
                return Err(AssertionError(msg).into());
            }
        };

        events.push(Event { data, start })
    }

    assert_that!("sequence event end", read.offset == end_offset, read.offset)?;
    Ok(events)
}

pub fn write_events<W: Write>(write: &mut W, anim_def: &AnimDef, events: &[Event]) -> Result<()> {
    for event in events {
        let event_type = event_type(event);
        let (start_offset, start_time) = match &event.start {
            None => (StartOffset::Animation as u8, 0.0),
            Some((StartOffset::Animation, start_time)) => {
                (StartOffset::Animation as u8, *start_time)
            }
            Some((StartOffset::Event, start_time)) => (StartOffset::Event as u8, *start_time),
            Some((StartOffset::Sequence, start_time)) => (StartOffset::Sequence as u8, *start_time),
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
    use EventData::*;
    match &event.data {
        Sound(_) => sound::Sound::INDEX,
        SoundNode(_) => sound_node::SoundNode::INDEX,
        LightState(_) => light_state::LightState::INDEX,
        LightAnimation(_) => light_animation::LightAnimation::INDEX,
        ObjectActiveState(_) => object_active_state::ObjectActiveState::INDEX,
        ObjectTranslateState(_) => object_translate_state::ObjectTranslateState::INDEX,
        ObjectScaleState(_) => object_scale_state::ObjectScaleState::INDEX,
        ObjectRotateState(_) => object_rotate_state::ObjectRotateState::INDEX,
        ObjectMotion(_) => object_motion::ObjectMotion::INDEX,
        ObjectMotionFromTo(_) => object_motion_from_to::ObjectMotionFromTo::INDEX,
        ObjectOpacityState(_) => object_opacity_state::ObjectOpacityState::INDEX,
        ObjectOpacityFromTo(_) => object_opacity_from_to::ObjectOpacityFromTo::INDEX,
        ObjectAddChild(_) => object_add_child::ObjectAddChild::INDEX,
        ObjectCycleTexture(_) => object_cycle_texture::ObjectCycleTexture::INDEX,
        ObjectConnector(_) => object_connector::ObjectConnector::INDEX,
        CallObjectConnector(_) => call_object_connector::CallObjectConnector::INDEX,
        CallSequence(_) => sequence::CallSequence::INDEX,
        StopSequence(_) => sequence::StopSequence::INDEX,
        CallAnimation(_) => call_animation::CallAnimation::INDEX,
        StopAnimation(_) => animation::StopAnimation::INDEX,
        ResetAnimation(_) => animation::ResetAnimation::INDEX,
        InvalidateAnimation(_) => animation::InvalidateAnimation::INDEX,
        FogState(_) => fog_state::FogState::INDEX,
        Loop(_) => control_flow::Loop::INDEX,
        If(_) => control_flow::If::INDEX,
        Else(_) => control_flow::Else::INDEX,
        Elif(_) => control_flow::ElseIf::INDEX,
        Endif(_) => control_flow::EndIf::INDEX,
        Callback(_) => control_flow::Callback::INDEX,
        FrameBufferEffectColorFromTo(_) => fbfx_color_from_to::FrameBufferEffectColor::INDEX,
        DetonateWeapon(_) => detonate_weapon::DetonateWeapon::INDEX,
        PufferState(_) => puffer_state::PufferState::INDEX,
        ObjectMotionSIScript(_) => object_motion_si_script::ObjectMotionSiScript::INDEX,
    }
}

fn size_event(event: &Event) -> u32 {
    use EventData::*;
    let size = match &event.data {
        Sound(_) => sound::Sound::SIZE,
        SoundNode(_) => sound_node::SoundNode::SIZE,
        LightState(_) => light_state::LightState::SIZE,
        LightAnimation(_) => light_animation::LightAnimation::SIZE,
        ObjectActiveState(_) => object_active_state::ObjectActiveState::SIZE,
        ObjectTranslateState(_) => object_translate_state::ObjectTranslateState::SIZE,
        ObjectScaleState(_) => object_scale_state::ObjectScaleState::SIZE,
        ObjectRotateState(_) => object_rotate_state::ObjectRotateState::SIZE,
        ObjectMotion(_) => object_motion::ObjectMotion::SIZE,
        ObjectMotionFromTo(_) => object_motion_from_to::ObjectMotionFromTo::SIZE,
        ObjectOpacityState(_) => object_opacity_state::ObjectOpacityState::SIZE,
        ObjectOpacityFromTo(_) => object_opacity_from_to::ObjectOpacityFromTo::SIZE,
        ObjectAddChild(_) => object_add_child::ObjectAddChild::SIZE,
        ObjectCycleTexture(_) => object_cycle_texture::ObjectCycleTexture::SIZE,
        ObjectConnector(_) => object_connector::ObjectConnector::SIZE,
        CallObjectConnector(_) => call_object_connector::CallObjectConnector::SIZE,
        CallSequence(_) => sequence::CallSequence::SIZE,
        StopSequence(_) => sequence::StopSequence::SIZE,
        CallAnimation(_) => call_animation::CallAnimation::SIZE,
        StopAnimation(_) => animation::StopAnimation::SIZE,
        ResetAnimation(_) => animation::ResetAnimation::SIZE,
        InvalidateAnimation(_) => animation::InvalidateAnimation::SIZE,
        FogState(_) => fog_state::FogState::SIZE,
        Loop(_) => control_flow::Loop::SIZE,
        If(_) => control_flow::If::SIZE,
        Else(_) => control_flow::Else::SIZE,
        Elif(_) => control_flow::ElseIf::SIZE,
        Endif(_) => control_flow::EndIf::SIZE,
        Callback(_) => control_flow::Callback::SIZE,
        FrameBufferEffectColorFromTo(_) => fbfx_color_from_to::FrameBufferEffectColor::SIZE,
        DetonateWeapon(_) => detonate_weapon::DetonateWeapon::SIZE,
        PufferState(_) => puffer_state::PufferState::SIZE,
        ObjectMotionSIScript(script) => script.size(),
    };
    size + EventHeaderC::SIZE
}

fn write_event<W: Write>(write: &mut W, anim_def: &AnimDef, event: &Event) -> Result<()> {
    use EventData::*;
    match &event.data {
        Sound(data) => data.write(write, anim_def),
        SoundNode(data) => data.write(write, anim_def),
        LightState(data) => data.write(write, anim_def),
        LightAnimation(data) => data.write(write, anim_def),
        ObjectActiveState(data) => data.write(write, anim_def),
        ObjectTranslateState(data) => data.write(write, anim_def),
        ObjectScaleState(data) => data.write(write, anim_def),
        ObjectRotateState(data) => data.write(write, anim_def),
        ObjectMotion(data) => data.write(write, anim_def),
        ObjectMotionFromTo(data) => data.write(write, anim_def),
        ObjectOpacityState(data) => data.write(write, anim_def),
        ObjectOpacityFromTo(data) => data.write(write, anim_def),
        ObjectAddChild(data) => data.write(write, anim_def),
        ObjectCycleTexture(data) => data.write(write, anim_def),
        ObjectConnector(data) => data.write(write, anim_def),
        CallObjectConnector(data) => data.write(write, anim_def),
        CallSequence(data) => data.write(write, anim_def),
        StopSequence(data) => data.write(write, anim_def),
        CallAnimation(data) => data.write(write, anim_def),
        StopAnimation(data) => data.write(write, anim_def),
        ResetAnimation(data) => data.write(write, anim_def),
        InvalidateAnimation(data) => data.write(write, anim_def),
        FogState(data) => data.write(write, anim_def),
        Loop(data) => data.write(write, anim_def),
        If(data) => data.write(write, anim_def),
        Else(data) => data.write(write, anim_def),
        Elif(data) => data.write(write, anim_def),
        Endif(data) => data.write(write, anim_def),
        Callback(data) => data.write(write, anim_def),
        FrameBufferEffectColorFromTo(data) => data.write(write, anim_def),
        DetonateWeapon(data) => data.write(write, anim_def),
        PufferState(data) => data.write(write, anim_def),
        ObjectMotionSIScript(data) => data.write(write, anim_def),
    }
}

pub fn size_events(events: &[Event]) -> u32 {
    let mut size = 0;
    for event in events {
        size += size_event(event);
    }
    size
}
