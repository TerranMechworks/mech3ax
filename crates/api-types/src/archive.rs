//! Archive-based `*.zbd` data structures.
use crate::serde::bytes;
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::{Struct, Union};
use mech3ax_timestamp::DateTime;

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct ArchiveEntry {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub rename: Option<String>,
    pub flags: u32,
    pub info: ArchiveEntryInfo,
}

#[derive(Debug, Serialize, Deserialize, Union)]
pub enum ArchiveEntryInfo {
    Valid(ArchiveEntryInfoValid),
    Invalid(ArchiveEntryInfoInvalid),
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct ArchiveEntryInfoValid {
    pub comment: String,
    pub datetime: DateTime,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct ArchiveEntryInfoInvalid {
    #[serde(with = "bytes")]
    pub comment: Vec<u8>,
    pub filetime: u64,
}
