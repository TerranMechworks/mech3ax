pub mod mw;
pub mod pm;
pub mod rc;

use crate::serde::{bool_false, bool_true};
use crate::{fld, Color, Matrix, Range, Vec3};
use ::serde::{Deserialize, Serialize};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_metadata_proc_macro::Struct;
use mech3ax_types::impl_as_bytes;

fld! {
    struct Camera {
        clip: Range,
        fov: Range,
        focus_node_xy: i32,
        data_ptr: u32,
    }
}

fld! {
    struct Display {
        resolution_x: u32,
        resolution_y: u32,
        clear_color: Color,
        data_ptr: u32,
    }
}

fld! {
    struct Window {
        resolution_x: u32,
        resolution_y: u32,
        data_ptr: u32,
    }
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

fld! {
    struct Transformation {
        rotation: Vec3,
        translation: Vec3,
        matrix: Option<Matrix>,
    }
}

fld! {
    struct PartitionPg {
        x: i32,
        y: i32,
        z_min: f32,
        z_max: f32,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        z_mid: Option<f32>,
        nodes: Vec<u32>,
        ptr: u32,
    }
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

fld! {
    struct PartitionNg {
        x: i32,
        y: i32,
        z_min: f32,
        z_max: f32,
        nodes: Vec<PartitionValue>,
        ptr: u32,
    }
}

#[inline]
fn _true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Struct)]
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
    pub bbox_node: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub bbox_model: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub bbox_child: bool,
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
