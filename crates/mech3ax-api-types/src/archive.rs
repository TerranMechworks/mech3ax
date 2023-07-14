//! Archive-based `*.zbd` data structures.
use crate::serde::bytes;
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::RefStruct;

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct ArchiveEntry {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub rename: Option<String>,
    #[serde(with = "bytes")]
    pub garbage: Vec<u8>,
}
