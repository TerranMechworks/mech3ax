use crate::types::{Quaternion, Vec3};
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::RefStruct;

#[derive(Debug, Serialize, Deserialize, RefStruct)]
#[generic(Vec3 = "TVec3", Quaternion = "TQuaternion")]
pub struct MotionFrame {
    pub translation: Vec3,
    pub rotation: Quaternion,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct MotionPart {
    pub name: String,
    pub frames: Vec<MotionFrame>,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct Motion {
    pub loop_time: f32,
    pub parts: Vec<MotionPart>,
    pub frame_count: u32,
}
