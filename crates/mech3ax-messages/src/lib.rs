#![warn(clippy::all, clippy::cargo)]
#![allow(clippy::identity_op)]
mod size;

mod bin;
mod message_table;
mod pe;
mod read;
mod resources;
mod string_table;
mod zloc;

pub use read::read_messages;
