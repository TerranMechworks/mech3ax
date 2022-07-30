#![warn(clippy::all, clippy::cargo)]
mod size;

mod bin;
mod message_table;
mod pe;
mod read;
mod resources;
mod zloc;

use ::serde::{Deserialize, Serialize};

pub use read::read_messages;

#[derive(Debug, Serialize, Deserialize)]
pub struct Messages {
    pub language_id: u32,
    pub entries: Vec<(String, u32, String)>,
}
