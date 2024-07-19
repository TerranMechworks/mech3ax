use crate::serde::bytes::Bytes;
use ::serde::{Deserialize, Serialize};
use num_derive::FromPrimitive;
use std::num::NonZeroU32;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, FromPrimitive)]
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
    Two(Option<Bytes>),
    Five(Option<Bytes>),
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
