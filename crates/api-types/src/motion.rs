//! MW3 `motion.zbd` data structures.
use crate::{Quaternion, Vec3};
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::Struct;

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(generic(Vec3 = "TVec3", Quaternion = "TQuaternion"))]
pub struct MotionFrame {
    pub translation: Vec3,
    pub rotation: Quaternion,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct MotionPart {
    pub name: String,
    pub frames: Vec<MotionFrame>,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct Motion {
    pub loop_time: f32,
    pub parts: Vec<MotionPart>,
    pub frame_count: u32,
}
