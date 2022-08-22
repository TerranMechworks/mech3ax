use crate::sequence_event::Event;
use ::serde::{Deserialize, Serialize};
use mech3ax_api_types::serde::base64;
use mech3ax_api_types::Range;
use mech3ax_common::assert::AssertionError;
use mech3ax_common::{assert_that, Result};
use num_derive::FromPrimitive;

#[derive(Debug, Serialize, Deserialize, FromPrimitive, PartialEq, Copy, Clone)]
#[repr(u32)]
pub enum AnimActivation {
    WeaponHit = 0,
    CollideHit = 1,
    WeaponOrCollideHit = 2,
    OnCall = 3,
    OnStartup = 4,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Execution {
    ByRange(Range),
    ByZone,
    None,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NamePad {
    pub name: String,
    #[serde(with = "base64")]
    pub pad: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NamePtr {
    pub name: String,
    pub pointer: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NamePtrFlags {
    pub name: String,
    pub pointer: u32,
    pub flags: u32,
}

#[derive(Debug, Serialize, Deserialize, FromPrimitive, PartialEq)]
#[repr(u32)]
pub enum SeqActivation {
    Initial = 0,
    OnCall = 3,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrereqObject {
    pub name: String,
    pub required: bool,
    pub active: bool,
    pub pointer: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ActivationPrereq {
    Animation(String),
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

impl AnimDef {
    pub fn node_from_index(&self, index: usize, offset: u32) -> Result<String> {
        if let Some(nodes) = &self.nodes {
            assert_that!("node index", 1 <= index <= nodes.len(), offset)?;
            Ok(nodes[index - 1].name.clone())
        } else {
            let msg = format!(
                "Tried to look up node {}, but anim def has no nodes (at {})",
                index, offset
            );
            Err(AssertionError(msg).into())
        }
    }

    pub fn node_to_index(&self, name: &str) -> Result<usize> {
        if let Some(nodes) = &self.nodes {
            nodes
                .iter()
                .position(|node| node.name == name)
                .map(|pos| pos + 1)
                .ok_or_else(|| {
                    AssertionError(format!("Expected to find node '{}', but didn't", name)).into()
                })
        } else {
            let msg = format!(
                "Expected to find node '{}', but anim def has no nodes",
                name
            );
            Err(AssertionError(msg).into())
        }
    }

    pub fn sound_from_index(&self, index: usize, offset: u32) -> Result<String> {
        if let Some(sounds) = &self.static_sounds {
            assert_that!("sound index", 1 <= index <= sounds.len(), offset)?;
            Ok(sounds[index - 1].name.clone())
        } else {
            let msg = format!(
                "Tried to look up sound {}, but anim def has no sounds (at {})",
                index, offset
            );
            Err(AssertionError(msg).into())
        }
    }

    pub fn sound_to_index(&self, name: &str) -> Result<usize> {
        if let Some(sounds) = &self.static_sounds {
            sounds
                .iter()
                .position(|sound| sound.name == name)
                .map(|pos| pos + 1)
                .ok_or_else(|| {
                    AssertionError(format!("Expected to find sound '{}', but didn't", name)).into()
                })
        } else {
            let msg = format!(
                "Expected to find sound '{}', but anim def has no sounds",
                name
            );
            Err(AssertionError(msg).into())
        }
    }

    pub fn light_from_index(&self, index: usize, offset: u32) -> Result<String> {
        if let Some(lights) = &self.lights {
            assert_that!("light index", 1 <= index <= lights.len(), offset)?;
            Ok(lights[index - 1].name.clone())
        } else {
            let msg = format!(
                "Tried to look up light {}, but anim def has no lights (at {})",
                index, offset
            );
            Err(AssertionError(msg).into())
        }
    }

    pub fn light_to_index(&self, name: &str) -> Result<usize> {
        if let Some(lights) = &self.lights {
            lights
                .iter()
                .position(|light| light.name == name)
                .map(|pos| pos + 1)
                .ok_or_else(|| {
                    AssertionError(format!("Expected to find light '{}', but didn't", name)).into()
                })
        } else {
            let msg = format!(
                "Expected to find light '{}', but anim def has no lights",
                name
            );
            Err(AssertionError(msg).into())
        }
    }

    pub fn puffer_from_index(&self, index: usize, offset: u32) -> Result<String> {
        if let Some(puffers) = &self.puffers {
            assert_that!("puffer index", 1 <= index <= puffers.len(), offset)?;
            Ok(puffers[index - 1].name.clone())
        } else {
            let msg = format!(
                "Tried to look up puffer {}, but anim def has no puffers (at {})",
                index, offset
            );
            Err(AssertionError(msg).into())
        }
    }

    pub fn puffer_to_index(&self, name: &str) -> Result<usize> {
        if let Some(puffers) = &self.puffers {
            puffers
                .iter()
                .position(|puffer| puffer.name == name)
                .map(|pos| pos + 1)
                .ok_or_else(|| {
                    AssertionError(format!("Expected to find puffer '{}', but didn't", name)).into()
                })
        } else {
            let msg = format!(
                "Expected to find puffer '{}', but anim def has no puffers",
                name
            );
            Err(AssertionError(msg).into())
        }
    }
}
