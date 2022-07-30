#![warn(clippy::all, clippy::cargo)]

mod interp;
mod rfc3339;

use ::serde::{Deserialize, Serialize};
use time::OffsetDateTime;

pub use interp::{read_interp, write_interp};

#[derive(Serialize, Deserialize, Debug)]
pub struct Script {
    pub name: String,
    #[serde(with = "rfc3339")]
    pub last_modified: OffsetDateTime,
    pub lines: Vec<String>,
}

#[cfg(test)]
mod tests;
