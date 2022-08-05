#![warn(clippy::all, clippy::cargo)]
mod motion;

pub use motion::{read_motion, write_motion};
