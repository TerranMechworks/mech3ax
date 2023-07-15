//! Interpreter (`interp.zbd`) data structures.
use crate::serde::rfc3339;
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::Struct;
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct Script {
    pub name: String,
    #[serde(with = "rfc3339")]
    pub last_modified: OffsetDateTime,
    pub lines: Vec<String>,
}
