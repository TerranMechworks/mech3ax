use super::activation_prereq::ActivationPrerequisite;
use super::events::Event;
use super::support::{
    AnimRef, DynamicSoundRef, EffectRef, LightRef, NodeRef, ObjectRef, PufferRef, StaticSoundRef,
};
use crate::serde::{bytes, bytes_opt};
use crate::{Range, api, num, sum};
use mech3ax_timestamp::DateTime;

api! {
    /// `ANIMATION_DEFINITION_FILE` in an `ANIMATION_LIST`
    struct AnimDefFile {
        name: String,
        datetime: DateTime,
        #[serde(skip_serializing_if = "Option::is_none", default, with = "bytes_opt")]
        garbage: Option<Vec<u8>> = { None },
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

api! {
    struct NamePad : Val {
        name: String,
        #[serde(with = "bytes")]
        pad: Vec<u8>,
    }
}

api! {
    struct NamePtr : Val {
        name: String,
        pointer: u32,
    }
}

api! {
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

api! {
    struct ResetState {
        events: Vec<Event>,
        pointer: u32,
    }
}

api! {
    struct SeqDef {
        name: String,
        seq_state: SeqDefState,
        reset_state: SeqDefState,
        events: Vec<Event>,
        pointer: u32,
    }
}

api! {
    struct AnimDef {
        name: String,
        anim_name: String,
        anim_root_name: String,

        has_callbacks: bool,
        /// MW/PM only
        auto_reset_node_states: bool,
        /// PM only
        local_nodes_only: bool,
        /// MW/PM only
        proximity_damage: bool,

        /// PM only?
        active: bool = { true },
        /// RC only?
        low_priority: bool = { false },
        activation: AnimActivation,
        execution: Execution,
        network_log: Option<bool> = { None },
        save_log: Option<bool> = { None },
        reset_time: Option<f32> = { None },

        health: f32,
        activ_prereq_min_to_satisfy: u8,

        objects: Option<Vec<ObjectRef>> = { None },
        nodes: Option<Vec<NodeRef>> = { None },
        lights: Option<Vec<LightRef>> = { None },
        puffers: Option<Vec<PufferRef>> = { None },
        dynamic_sounds: Option<Vec<DynamicSoundRef>> = { None },
        static_sounds: Option<Vec<StaticSoundRef>> = { None },
        effects: Option<Vec<EffectRef>> = { None },
        activ_prereqs: Option<Vec<ActivationPrerequisite>> = { None },
        anim_refs: Option<Vec<AnimRef>> = { None },
        reset_state: Option<ResetState>,
        sequences: Vec<SeqDef>,
        ptrs: Option<AnimDefPtrs> = { None },
    }
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

api! {
    struct AnimDefPtrs {
        anim_hash: Option<u32> = { None },
        anim_root_hash: Option<u32> = { None },

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
