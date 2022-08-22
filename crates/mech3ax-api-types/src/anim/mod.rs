use crate::serde::base64;
use ::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AnimName {
    pub name: String,
    #[serde(with = "base64")]
    pub pad: Vec<u8>,
    pub unknown: u32,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct AnimMetadata {
    pub base_ptr: u32,
    pub world_ptr: u32,
    pub anim_names: Vec<AnimName>,
    pub anim_ptrs: Vec<AnimPtr>,
}
