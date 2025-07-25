//! MW3/PM `motion.zbd` data structures.
use crate::{Quaternion, Vec3, api};

api! {
    struct MotionFrame {
        translation: Vec3,
        rotation: Quaternion,
    }
}

api! {
    struct MotionPart {
        name: String,
        frames: Vec<MotionFrame>,
    }
}

api! {
    struct Motion {
        loop_time: f32,
        parts: Vec<MotionPart>,
        frame_count: u32,
    }
}
