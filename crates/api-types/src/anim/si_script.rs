use crate::serde::bytes;
use crate::{fld, Quaternion, Vec3};

fld! {
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

fld! {
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

fld! {
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

fld! {
    struct ObjectMotionSiFrame {
        start_time: f32,
        end_time: f32,
        translate: Option<TranslateData> = { None },
        rotate: Option<RotateData> = { None },
        scale: Option<ScaleData> = { None },
    }
}

fld! {
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
