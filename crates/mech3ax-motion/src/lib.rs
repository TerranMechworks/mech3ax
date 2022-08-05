#![warn(clippy::all, clippy::cargo)]
mod motion;

use ::serde::{Deserialize, Serialize};
use mech3ax_common::types::{Vec3, Vec4};

pub use motion::{read_motion, write_motion};

#[derive(Debug, Serialize, Deserialize)]
pub struct Frame {
    translation: Vec3,
    rotation: Vec4,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Motion {
    loop_time: f32,
    // need to preserve order
    parts: Vec<(String, Vec<Frame>)>,
    frame_count: u32,
}
