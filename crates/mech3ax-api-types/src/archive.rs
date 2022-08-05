use crate::serde::base64;
use ::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ArchiveEntry {
    pub name: String,
    #[serde(with = "base64")]
    pub garbage: Vec<u8>,
}
