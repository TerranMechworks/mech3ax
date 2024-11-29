#![warn(clippy::all, clippy::cargo)]
#![allow(clippy::identity_op)]
pub(crate) mod events;
mod types;

pub use events::{read_events, size_events, write_events};
