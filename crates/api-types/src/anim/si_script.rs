use crate::serde::bytes;
use crate::{Quaternion, Vec3, api};

api! {
    struct TranslateData {
        base: Vec3,
        delta: Vec3,
        garbage: u32,
        #[serde(with = "bytes")]
        spline_x: Vec<u8>,
        #[serde(with = "bytes")]
        spline_y: Vec<u8>,
        #[serde(with = "bytes")]
        spline_z: Vec<u8>,
    }
}

api! {
    struct RotateData {
        base: Quaternion,
        delta: Vec3,
        #[serde(with = "bytes")]
        spline_x: Vec<u8>,
        #[serde(with = "bytes")]
        spline_y: Vec<u8>,
        #[serde(with = "bytes")]
        spline_z: Vec<u8>,
    }
}

api! {
    struct ScaleData {
        base: Vec3,
        delta: Vec3,
        garbage: u32,
        #[serde(with = "bytes")]
        spline_x: Vec<u8>,
        #[serde(with = "bytes")]
        spline_y: Vec<u8>,
        #[serde(with = "bytes")]
        spline_z: Vec<u8>,
    }
}

api! {
    struct ObjectMotionSiFrame {
        start_time: f32,
        end_time: f32,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        translate: Option<TranslateData> = { None },
        #[serde(skip_serializing_if = "Option::is_none", default)]
        rotate: Option<RotateData> = { None },
        #[serde(skip_serializing_if = "Option::is_none", default)]
        scale: Option<ScaleData> = { None },
    }
}

api! {
    struct SiScript {
        script_name: String,
        object_name: String,
        frames: Vec<ObjectMotionSiFrame>,
        /// Never set in PM
        spline_interp: bool,
        script_name_ptr: u32,
        object_name_ptr: u32,
        script_data_ptr: u32,
    }
}
