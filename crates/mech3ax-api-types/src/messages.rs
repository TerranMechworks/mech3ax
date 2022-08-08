use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::RefStruct;

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct MessageEntry {
    pub key: String,
    pub id: u32,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct Messages {
    pub language_id: u32,
    pub entries: Vec<MessageEntry>,
}
