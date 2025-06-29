//! MW3/PM `motion.zbd` data structures.
use crate::{fld, Quaternion, Vec3};

fld! {
    struct MotionFrame {
        translation: Vec3,
        rotation: Quaternion,
    }
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
