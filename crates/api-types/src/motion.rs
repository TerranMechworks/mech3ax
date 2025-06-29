//! MW3/PM `motion.zbd` data structures.
use crate::{fld, Quaternion, Vec3};
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::Struct;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Struct)]
#[dotnet(generic(Vec3 = "TVec3", Quaternion = "TQuaternion"))]
pub struct MotionFrame {
    pub translation: Vec3,
    pub rotation: Quaternion,
}

fld! {
    struct MotionPart {
        name: String,
        frames: Vec<MotionFrame>,
    }
}

fld! {
    struct Motion {
        loop_time: f32,
        parts: Vec<MotionPart>,
        frame_count: u32,
    }
}
