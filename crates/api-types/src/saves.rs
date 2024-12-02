//! Work in progress.
use crate::serde::bytes::Bytes;
use ::serde::{Deserialize, Serialize};
use mech3ax_types::primitive_enum;
use std::num::NonZeroU32;

primitive_enum! {
    #[derive(Serialize, Deserialize)]
    pub enum ActivationStatus: u8 {
        Unk1 = 1,
        // Unk2 = 2,
        Executed = 3,
        Invalid = 4,
        // Unk5 = 5,
        // Unk6 = 6,
    }
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
