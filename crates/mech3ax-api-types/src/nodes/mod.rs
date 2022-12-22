pub mod cs;
pub mod mw;
pub mod pm;
pub mod rc;

use crate::serde::{bool_false, bool_true};
use crate::static_assert_size;
use crate::types::{Color, Matrix, Range, Vec3};
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::{RefStruct, ValStruct};

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct Camera {
    pub clip: Range,
    pub fov: Range,
    pub focus_node_xy: i32,
    pub data_ptr: u32,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct Display {
    pub resolution_x: u32,
    pub resolution_y: u32,
    pub clear_color: Color,
    pub data_ptr: u32,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct Window {
    pub resolution_x: u32,
    pub resolution_y: u32,
    pub data_ptr: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ValStruct)]
#[repr(C)]
pub struct AreaPartition {
    pub x: i32,
    pub y: i32,
}
static_assert_size!(AreaPartition, 8);

impl AreaPartition {
    pub const DEFAULT: Self = Self { x: -1, y: -1 };
    pub const ZERO: Self = Self { x: 0, y: 0 };
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ValStruct)]
pub struct Area {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Area {
    pub const fn x_count(&self, size: i32) -> i32 {
        (self.right - self.left) / size
    }

    pub const fn y_count(&self, size: i32) -> i32 {
        (self.bottom - self.top) / size
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, RefStruct)]
#[repr(C)]
pub struct BoundingBox {
    pub a: Vec3,
    pub b: Vec3,
}

impl BoundingBox {
    pub const EMPTY: Self = Self {
        a: Vec3::DEFAULT,
        b: Vec3::DEFAULT,
    };
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct Transformation {
    pub rotation: Vec3,
    pub translation: Vec3,
    pub matrix: Option<Matrix>,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct PartitionPg {
    pub x: i32,
    pub y: i32,
    pub z_min: f32,
    pub z_max: f32,
    pub nodes: Vec<u32>,
    pub ptr: u32,
}

#[derive(Debug, Serialize, Deserialize, RefStruct, Clone, PartialEq)]
#[repr(C)]
pub struct PartitionValue {
    pub index: u32,
    pub z_min: f32,
    pub z_max: f32,
}
static_assert_size!(PartitionValue, 12);

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct PartitionNg {
    pub x: i32,
    pub y: i32,
    pub z_min: f32,
    pub z_max: f32,
    pub nodes: Vec<PartitionValue>,
    pub ptr: u32,
}

#[inline]
fn _true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct NodeFlags {
    #[serde(skip_serializing_if = "bool_true", default = "_true")]
    pub active: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub altitude_surface: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub intersect_surface: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub intersect_bbox: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub landmark: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub unk08: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub has_mesh: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub unk10: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub terrain: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub can_modify: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub clip_to: bool,
    #[serde(skip_serializing_if = "bool_true", default = "_true")]
    pub tree_valid: bool,
    #[serde(skip_serializing_if = "bool_true", default = "_true")]
    pub id_zone_check: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub unk25: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub unk28: bool,
}
