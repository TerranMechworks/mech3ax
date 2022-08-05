use crate::serde::rfc3339;
use ::serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct Script {
    pub name: String,
    #[serde(with = "rfc3339")]
    pub last_modified: OffsetDateTime,
    pub lines: Vec<String>,
}
