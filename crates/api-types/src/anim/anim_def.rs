use super::activation_prereq::ActivationPrerequisite;
use super::events::Event;
use super::support::{
    AnimRef, DynamicSoundRef, EffectRef, LightRef, NodeRef, ObjectRef, PufferRef, StaticSoundRef,
};
use crate::serde::{bool_false, bool_true, bytes};
use crate::Range;
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::{Enum, Struct, Union};
use mech3ax_timestamp::DateTime;
use mech3ax_types::primitive_enum;

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct AnimDefName {
    pub file_name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub rename: Option<String>,
}

/// `ANIMATION_DEFINITION_FILE` in an `ANIMATION_LIST`
#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct AnimDefFile {
    pub name: String,
    pub datetime: DateTime,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub hash: Option<u32>,
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
    pub enum SeqDefState: u8 {
        Initial = 0,
        OnCall = 3,
    }
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct ResetState {
    pub events: Vec<Event>,
    pub pointer: u32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct SeqDef {
    pub name: String,
    pub seq_state: SeqDefState,
    pub reset_state: SeqDefState,
    pub events: Vec<Event>,
    pub pointer: u32,
}

#[inline]
fn _true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct AnimDef {
    pub name: String,
    pub anim_name: String,
    pub anim_root_name: String,
    pub file_name: String,

    pub has_callbacks: bool,
    /// MW/PM only
    pub auto_reset_node_states: bool,
    /// PM only
    pub local_nodes_only: bool,
    /// MW/PM only
    pub proximity_damage: bool,

    /// PM only?
    #[serde(skip_serializing_if = "bool_true", default = "_true")]
    pub active: bool,
    /// RC only?
    #[serde(skip_serializing_if = "bool_false", default)]
    pub low_priority: bool,
    pub activation: AnimActivation,
    pub execution: Execution,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub network_log: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub save_log: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub reset_time: Option<f32>,

    pub health: f32,
    pub activ_prereq_min_to_satisfy: u8,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub objects: Option<Vec<ObjectRef>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub nodes: Option<Vec<NodeRef>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub lights: Option<Vec<LightRef>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub puffers: Option<Vec<PufferRef>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub dynamic_sounds: Option<Vec<DynamicSoundRef>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub static_sounds: Option<Vec<StaticSoundRef>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub effects: Option<Vec<EffectRef>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub activ_prereqs: Option<Vec<ActivationPrerequisite>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub anim_refs: Option<Vec<AnimRef>>,

    pub reset_state: Option<ResetState>,
    pub sequences: Vec<SeqDef>,

    pub anim_ptr: u32,
    pub anim_root_ptr: u32,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub anim_hash: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub anim_root_hash: Option<u32>,

    pub seq_defs_ptr: u32,
    // PN only?
    pub reset_state_ptr: u32,
    pub objects_ptr: u32,
    pub nodes_ptr: u32,
    pub lights_ptr: u32,
    pub puffers_ptr: u32,
    pub dynamic_sounds_ptr: u32,
    pub static_sounds_ptr: u32,
    pub effects_ptr: u32,
    pub activ_prereqs_ptr: u32,
    pub anim_refs_ptr: u32,
}
