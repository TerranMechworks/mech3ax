//! Localisation data structures.
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::Struct;

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct MessageEntry {
    pub key: String,
    pub id: u32,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(partial, namespace = "Mech3DotNet.Zbd")]
pub struct Messages {
    pub language_id: u32,
    pub entries: Vec<MessageEntry>,
}
