use crate::types::{Vec3, Vec4};
use ::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MotionFrame {
    pub translation: Vec3,
    pub rotation: Vec4,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Motion {
    pub loop_time: f32,
    // need to preserve order
    pub parts: Vec<(String, Vec<MotionFrame>)>,
    pub frame_count: u32,
}