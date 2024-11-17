//! Interpreter (`interp.zbd`) data structures.
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::Struct;
use mech3ax_timestamp::DateTime;

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct Script {
    pub name: String,
    pub last_modified: DateTime,
    pub lines: Vec<String>,
}
