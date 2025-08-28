pub mod cs;
pub mod mw;
pub mod pm;
pub mod rc;

use crate::serde::{bool_false, bool_true};
use crate::{Color, Matrix, Range, Vec3};
use ::serde::{Deserialize, Serialize};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_metadata_proc_macro::Struct;
use mech3ax_types::impl_as_bytes;

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct Camera {
    pub clip: Range,
    pub fov: Range,
    pub focus_node_xy: i32,
    pub data_ptr: u32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct Display {
    pub resolution_x: u32,
    pub resolution_y: u32,
    pub clear_color: Color,
    pub data_ptr: u32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct Window {
    pub resolution_x: u32,
    pub resolution_y: u32,
    pub data_ptr: u32,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    NoUninit,
    AnyBitPattern,
    Struct,
    Default,
)]
#[dotnet(val_struct)]
#[repr(C)]
pub struct AreaPartition {
    pub x: i32,
    pub y: i32,
}
impl_as_bytes!(AreaPartition, 8);

impl AreaPartition {
    pub const DEFAULT: Self = Self { x: -1, y: -1 };
    pub const ZERO: Self = Self { x: 0, y: 0 };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Struct)]
#[dotnet(val_struct)]
pub struct Area {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Area {
    #[inline]
    pub const fn x_count(&self, size: i32) -> i32 {
        (self.right - self.left) / size
    }

    #[inline]
    pub const fn y_count(&self, size: i32) -> i32 {
        (self.bottom - self.top) / size
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Serialize, Deserialize, NoUninit, AnyBitPattern, Struct, Default,
)]
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

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct Transformation {
    pub rotation: Vec3,
    pub translation: Vec3,
    pub matrix: Option<Matrix>,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct PartitionPg {
    pub x: i32,
    pub y: i32,
    pub z_min: f32,
    pub z_max: f32,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub z_mid: Option<f32>,
    pub nodes: Vec<u32>,
    pub ptr: u32,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Serialize, Deserialize, NoUninit, AnyBitPattern, Struct,
)]
#[repr(C)]
pub struct PartitionValue {
    pub index: u32,
    pub z_min: f32,
    pub z_max: f32,
}
impl_as_bytes!(PartitionValue, 12);

#[derive(Debug, Serialize, Deserialize, Struct)]
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

#[derive(Debug, Serialize, Deserialize, Struct)]
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
