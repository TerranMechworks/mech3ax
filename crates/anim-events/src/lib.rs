#![warn(clippy::all, clippy::cargo)]
#![allow(clippy::identity_op)]
pub(crate) mod events;
mod mw;
mod types;
mod utils;

pub use mw::{read_events, size_events, write_events};
