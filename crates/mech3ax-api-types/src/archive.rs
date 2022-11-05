use crate::serde::base64;
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::RefStruct;

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct ArchiveEntry {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub rename: Option<String>,
    #[serde(with = "base64")]
    pub garbage: Vec<u8>,
}
