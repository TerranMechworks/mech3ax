use crate::serde::bytes;
use crate::{Quaternion, Vec3};
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::Struct;

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct TranslateData {
    pub base: Vec3,
    pub delta: Vec3,
    pub garbage: u32,
    #[serde(with = "bytes")]
    pub spline_x: Vec<u8>,
    #[serde(with = "bytes")]
    pub spline_y: Vec<u8>,
    #[serde(with = "bytes")]
    pub spline_z: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct RotateData {
    pub base: Quaternion,
    pub delta: Vec3,
    #[serde(with = "bytes")]
    pub spline_x: Vec<u8>,
    #[serde(with = "bytes")]
    pub spline_y: Vec<u8>,
    #[serde(with = "bytes")]
    pub spline_z: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct ScaleData {
    pub base: Vec3,
    pub delta: Vec3,
    pub garbage: u32,
    #[serde(with = "bytes")]
    pub spline_x: Vec<u8>,
    #[serde(with = "bytes")]
    pub spline_y: Vec<u8>,
    #[serde(with = "bytes")]
    pub spline_z: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct ObjectMotionSiFrame {
    pub start_time: f32,
    pub end_time: f32,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub translate: Option<TranslateData>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub rotate: Option<RotateData>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub scale: Option<ScaleData>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Struct)]
pub struct SiScript {
    pub script_name: String,
    pub object_name: String,
    pub frames: Vec<ObjectMotionSiFrame>,
    /// Never set in PM
    pub spline_interp: bool,
    pub script_name_ptr: u32,
    pub object_name_ptr: u32,
    pub script_data_ptr: u32,
}
