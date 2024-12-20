pub mod events;

use crate::serde::bytes;
use crate::Range;
use ::serde::{Deserialize, Serialize};
use events::Event;
use mech3ax_metadata_proc_macro::{Enum, Struct, Union};
use mech3ax_types::primitive_enum;

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct AnimName {
    pub name: String,
    #[serde(with = "bytes")]
    pub pad: Vec<u8>,
    pub unknown: u32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
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
    pub seq_defs_ptr: u32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct AnimMetadata {
    pub base_ptr: u32,
    pub world_ptr: u32,
    pub anim_names: Vec<AnimName>,
    pub anim_ptrs: Vec<AnimPtr>,
}

primitive_enum! {
    #[derive(Serialize, Deserialize, Enum)]
    pub enum AnimActivation: u8 {
        WeaponHit = 0,
        CollideHit = 1,
        WeaponOrCollideHit = 2,
        OnCall = 3,
        OnStartup = 4,
    }
}

#[derive(Debug, Serialize, Deserialize, Union)]
pub enum Execution {
    ByRange(Range),
    ByZone,
    None,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(val_struct)]
pub struct NamePad {
    pub name: String,
    #[serde(with = "bytes")]
    pub pad: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(val_struct)]
pub struct NamePtr {
    pub name: String,
    pub pointer: u32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(val_struct)]
pub struct NamePtrFlags {
    pub name: String,
    pub pointer: u32,
    pub flags: u32,
}

primitive_enum! {
    #[derive(Serialize, Deserialize, Enum)]
    pub enum SeqActivation: u32 {
        Initial = 0,
        OnCall = 3,
    }
}

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(val_struct)]
pub struct PrereqAnimation {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(val_struct)]
pub struct PrereqObject {
    pub name: String,
    pub required: bool,
    pub active: bool,
    pub pointer: u32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(val_struct)]
pub struct PrereqParent {
    pub name: String,
    pub required: bool,
    pub active: bool,
    pub pointer: u32,
}

#[derive(Debug, Serialize, Deserialize, Union)]
pub enum ActivationPrereq {
    Animation(PrereqAnimation),
    Parent(PrereqParent),
    Object(PrereqObject),
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct ResetState {
    pub events: Vec<Event>,
    pub pointer: u32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct SeqDef {
    pub name: String,
    pub activation: SeqActivation,
    pub events: Vec<Event>,
    pub pointer: u32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
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

    pub reset_state: Option<ResetState>,
    pub sequences: Vec<SeqDef>,
}
