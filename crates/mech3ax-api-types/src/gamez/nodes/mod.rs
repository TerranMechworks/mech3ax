pub mod mw;
pub mod pm;

use crate::serde::bool_false;
use crate::static_assert_size;
use crate::types::{Matrix, Vec3};
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::{RefStruct, ValStruct};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, ValStruct)]
#[repr(C)]
pub struct AreaPartition {
    pub x: i32,
    pub y: i32,
}
static_assert_size!(AreaPartition, 8);

impl AreaPartition {
    pub const DEFAULT_MW: Self = Self { x: -1, y: -1 };
    pub const DEFAULT_PM: Self = Self { x: -1, y: 0 };
    pub const ZERO: Self = Self { x: 0, y: 0 };
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, ValStruct)]
pub struct Area {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
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
pub struct Partition {
    pub x: i32,
    pub y: i32,
    pub z_min: f32,
    pub z_max: f32,
    pub z_mid: f32,
    pub nodes: Vec<u32>,
    pub ptr: u32,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct NodeFlags {
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
    #[serde(skip_serializing_if = "bool_false", default)]
    pub unk25: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub unk28: bool,
}
