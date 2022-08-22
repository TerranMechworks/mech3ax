mod events;

use crate::serde::base64;
use crate::Range;
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::{Enum, RefStruct, Union, ValStruct};
use num_derive::FromPrimitive;

pub use events::*;

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct AnimName {
    pub name: String,
    #[serde(with = "base64")]
    pub pad: Vec<u8>,
    pub unknown: u32,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct AnimPtr {
    pub file_name: String,

    pub anim_ptr: u32,
    pub anim_root_ptr: u32,
    pub objects_ptr: u32,
    pub nodes_ptr: u32,
    pub lights_ptr: u32,
    pub puffers_ptr: u32,
    pub dynamic_sounds_ptr: u32,
    pub static_sounds_ptr: u32,
    pub activ_prereqs_ptr: u32,
    pub anim_refs_ptr: u32,
    pub reset_state_ptr: u32,
    pub reset_state_events_ptr: u32,
    pub seq_defs_ptr: u32,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct AnimMetadata {
    pub base_ptr: u32,
    pub world_ptr: u32,
    pub anim_names: Vec<AnimName>,
    pub anim_ptrs: Vec<AnimPtr>,
}

#[derive(Debug, Serialize, Deserialize, FromPrimitive, PartialEq, Clone, Copy, Enum)]
#[repr(u32)]
pub enum AnimActivation {
    WeaponHit = 0,
    CollideHit = 1,
    WeaponOrCollideHit = 2,
    OnCall = 3,
    OnStartup = 4,
}

#[derive(Debug, Serialize, Deserialize, Union)]
pub enum Execution {
    ByRange(Range),
    ByZone,
    None,
}

#[derive(Debug, Serialize, Deserialize, ValStruct)]
pub struct NamePad {
    pub name: String,
    #[serde(with = "base64")]
    pub pad: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, ValStruct)]
pub struct NamePtr {
    pub name: String,
    pub pointer: u32,
}

#[derive(Debug, Serialize, Deserialize, ValStruct)]
pub struct NamePtrFlags {
    pub name: String,
    pub pointer: u32,
    pub flags: u32,
}

#[derive(Debug, Serialize, Deserialize, FromPrimitive, PartialEq, Enum)]
#[repr(u32)]
pub enum SeqActivation {
    Initial = 0,
    OnCall = 3,
}

#[derive(Debug, Serialize, Deserialize, ValStruct)]
pub struct PrereqObject {
    pub name: String,
    pub required: bool,
    pub active: bool,
    pub pointer: u32,
}

#[derive(Debug, Serialize, Deserialize, ValStruct)]
pub struct PrereqAnimation {
    pub name: String,
}

// TODO: this will be broken in C#?
#[derive(Debug, Serialize, Deserialize, Union)]
pub enum ActivationPrereq {
    Animation(PrereqAnimation),
    Parent(PrereqObject),
    Object(PrereqObject),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SeqDef {
    pub name: String,
    pub activation: SeqActivation,
    pub events: Vec<Event>,
    pub pointer: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnimDef {
    pub name: String,
    pub anim_name: NamePad,
    pub anim_root: NamePad,
    pub file_name: String,

    pub auto_reset_node_states: bool, // = True
    pub activation: AnimActivation,
    pub execution: Execution,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub network_log: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub save_log: Option<bool>,
    pub has_callbacks: bool, // = False
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub reset_time: Option<f32>,
    pub health: f32,            // = 0.0
    pub proximity_damage: bool, // = True
    pub activ_prereq_min_to_satisfy: u8,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub objects: Option<Vec<NamePad>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub nodes: Option<Vec<NamePtr>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub lights: Option<Vec<NamePtr>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub puffers: Option<Vec<NamePtrFlags>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub dynamic_sounds: Option<Vec<NamePtr>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub static_sounds: Option<Vec<NamePad>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub activ_prereqs: Option<Vec<ActivationPrereq>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub anim_refs: Option<Vec<NamePad>>,

    pub reset_sequence: Option<SeqDef>,
    pub sequences: Vec<SeqDef>,
}
