#![warn(clippy::all, clippy::cargo)]
#![allow(clippy::identity_op)]
pub(crate) mod common;
pub(crate) mod events;
pub mod mw;
pub mod pm;
pub mod rc;
pub mod si_script;
mod types;
mod utils;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::{EventData, StartOffset};
use mech3ax_types::primitive_enum;
use mech3ax_types::{impl_as_bytes, Maybe};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct EventHeaderC {
    event_type: Maybe<u8, EventType>,
    start_offset: Maybe<u8, StartOffset>,
    pad: u16,
    size: u32,
    start_time: f32,
}
impl_as_bytes!(EventHeaderC, 12);

macro_rules! events {
    (
        $(
            #[doc = $doc:literal]
            $index:literal => $name:ident,
        )+
    ) => {
        primitive_enum! {
            pub(crate) enum EventType: u8 {
                $(#[doc = $doc]
                $name = $index,)+
            }
        }

        fn event_type(data: &EventData) -> Maybe<u8, EventType> {
            match data {
                $(EventData::$name(_) => EventType::$name.maybe(),)+
            }
        }
    };
}

events! {
    // --------- 0
    ///SOUND
    1 => Sound,
    ///SOUND_NODE
    2 => SoundNode,
    ///EFFECT
    3 => Effect,
    ///LIGHT_STATE
    4 => LightState,
    ///LIGHT_ANIMATION
    5 => LightAnimation,
    ///OBJECT_ACTIVE_STATE
    6 => ObjectActiveState,
    ///OBJECT_TRANSLATE_STATE
    7 => ObjectTranslateState,
    ///OBJECT_SCALE_STATE
    8 => ObjectScaleState,
    ///OBJECT_ROTATE_STATE
    9 => ObjectRotateState,
    ///OBJECT_MOTION
    10 => ObjectMotion,
    ///OBJECT_MOTION_FROM_TO
    11 => ObjectMotionFromTo,
    ///OBJECT_MOTION_SI_SCRIPT
    12 => ObjectMotionSiScript,
    ///OBJECT_OPACITY_STATE
    13 => ObjectOpacityState,
    ///OBJECT_OPACITY_FROM_TO
    14 => ObjectOpacityFromTo,
    ///OBJECT_ADD_CHILD
    15 => ObjectAddChild,
    ///OBJECT_DELETE_CHILD
    16 => ObjectDeleteChild,
    ///OBJECT_CYCLE_TEXTURE
    17 => ObjectCycleTexture,
    ///OBJECT_CONNECTOR
    18 => ObjectConnector,
    ///CALL_OBJECT_CONNECTOR
    19 => CallObjectConnector,
    ///CAMERA_STATE,
    20 => CameraState,
    ///CAMERA_FROM_TO,
    21 => CameraFromTo,
    ///CALL_SEQUENCE
    22 => CallSequence,
    ///STOP_SEQUENCE
    23 => StopSequence,
    ///CALL_ANIMATION
    24 => CallAnimation,
    ///STOP_ANIMATION
    25 => StopAnimation,
    ///RESET_ANIMATION
    26 => ResetAnimation,
    ///INVALIDATE_ANIMATION
    27 => InvalidateAnimation,
    ///FOG_STATE
    28 => FogState,
    // --------- 29
    ///LOOP
    30 => Loop,
    ///IF
    31 => If,
    ///ELSE
    32 => Else,
    ///ELSEIF
    33 => Elseif,
    ///ENDIF
    34 => Endif,
    ///CALLBACK
    35 => Callback,
    ///FBFX_COLOR_FROM_TO
    36 => FbfxColorFromTo,
    ///FBFX_CSINWAVE_FROM_TO
    37 => FbfxCsinwaveFromTo,
    // --------- 38
    ///ANIM_VERBOSE
    39 => AnimVerbose,
    // --------- 40
    ///DETONATE_WEAPON
    41 => DetonateWeapon, // MW/PM only
    ///PUFFER_STATE
    42 => PufferState, // MW/PM only
}
