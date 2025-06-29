//! Localisation data structures.
use crate::fld;
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::Struct;

fld! {
    struct MessageEntry {
        key: String,
        id: u32,
        value: String,
    }
}

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(partial, namespace = "Mech3DotNet.Zbd")]
pub struct Messages {
    pub language_id: u32,
    pub entries: Vec<MessageEntry>,
}
