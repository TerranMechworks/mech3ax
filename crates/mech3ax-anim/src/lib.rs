#![warn(clippy::all, clippy::cargo)]
#![allow(clippy::identity_op)]
mod activation_prereq;
mod anim_def;
mod parse;
mod sequence_event;
mod support;
mod types;

pub use parse::{read_anim, write_anim, AnimMetadata};
pub use types::AnimDef;
