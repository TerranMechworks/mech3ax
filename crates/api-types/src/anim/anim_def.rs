use super::activation_prereq::ActivationPrerequisite;
use super::events::Event;
use super::support::{
    AnimRef, DynamicSoundRef, EffectRef, LightRef, NodeRef, ObjectRef, PufferRef, StaticSoundRef,
};
use crate::serde::{bool_false, bool_true, bytes};
use crate::{fld, num, sum, Range};
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::Struct;
use mech3ax_timestamp::DateTime;

fld! {
    /// `ANIMATION_DEFINITION_FILE` in an `ANIMATION_LIST`
    struct AnimDefFile {
        name: String,
        datetime: DateTime,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        hash: Option<u32>,
    }
}

num! {
    enum AnimActivation: u8 {
        WeaponHit = 0,
        CollideHit = 1,
        WeaponOrCollideHit = 2,
        OnCall = 3,
        OnStartup = 4,
    }
}

sum! {
    enum Execution {
        ByRange(Range),
        ByZone,
        None,
    }
}

fld! {
    struct NamePad : Val {
        name: String,
        #[serde(with = "bytes")]
        pad: Vec<u8>,
    }
}

fld! {
    struct NamePtr : Val {
        name: String,
        pointer: u32,
    }
}

fld! {
    struct NamePtrFlags : Val {
        name: String,
        pointer: u32,
        flags: u32,
    }
}

num! {
    enum SeqDefState: u8 {
        Initial = 0,
        OnCall = 3,
    }
}

fld! {
    struct ResetState {
        events: Vec<Event>,
        pointer: u32,
    }
}

fld! {
    struct SeqDef {
        name: String,
        seq_state: SeqDefState,
        reset_state: SeqDefState,
        events: Vec<Event>,
        pointer: u32,
    }
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

    #[serde(default)]
    pub ptrs: Option<AnimDefPtrs>,
}

impl AnimDef {
    pub fn file_name(&self) -> String {
        let name = self.name.strip_suffix(".flt").unwrap_or(&self.name);
        let anim_name = self
            .anim_name
            .strip_suffix(".flt")
            .unwrap_or(&self.anim_name);
        let anim_root_name = self
            .anim_root_name
            .strip_suffix(".flt")
            .unwrap_or(&self.anim_root_name);

        if name != anim_root_name {
            format!("{}-{}-{}", name, anim_name, anim_root_name)
        } else {
            format!("{}-{}", name, anim_name)
        }
    }
}

fld! {
    struct AnimDefPtrs {
        #[serde(skip_serializing_if = "Option::is_none", default)]
        anim_hash: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        anim_root_hash: Option<u32>,

        seq_defs_ptr: u32,
        objects_ptr: u32,
        nodes_ptr: u32,
        lights_ptr: u32,
        dynamic_sounds_ptr: u32,
        static_sounds_ptr: u32,
        activ_prereqs_ptr: u32,
        anim_refs_ptr: u32,

        // MW/RC only, not PM
        anim_ptr: u32,
        // MW/RC only, not PM
        anim_root_ptr: u32,
        puffers_ptr: u32,
        // RC only, not MW/PM
        effects_ptr: u32,
        // PM only, not MW/RC
        reset_state_ptr: u32,
    }
}
