use crate::types::{Quaternion, Vec3};
use ::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MotionFrame {
    pub translation: Vec3,
    pub rotation: Quaternion,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MotionPart {
    pub name: String,
    pub frames: Vec<MotionFrame>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Motion {
    pub loop_time: f32,
    pub parts: Vec<MotionPart>,
    pub frame_count: u32,
}
