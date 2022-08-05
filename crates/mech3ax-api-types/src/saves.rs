use crate::serde::base64_opt;
use ::serde::{Deserialize, Serialize};
use num_derive::FromPrimitive;
use std::num::NonZeroU32;

#[derive(Debug, Serialize, Deserialize, PartialEq, FromPrimitive, Copy, Clone)]
#[repr(u8)]
pub enum ActivationStatus {
    Unk1 = 1,
    // Unk2 = 2,
    Executed = 3,
    Invalid = 4,
    // Unk5 = 5,
    // Unk6 = 6,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ActivationType {
    One,
    Two(#[serde(with = "base64_opt")] Option<Vec<u8>>),
    Five(#[serde(with = "base64_opt")] Option<Vec<u8>>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnimActivation {
    pub name: String,
    pub node_index: Option<i32>,
    pub status: ActivationStatus,
    pub type_: ActivationType,
    pub node_states: Vec<Vec<u8>>,
    pub ptr: Option<NonZeroU32>,
}
