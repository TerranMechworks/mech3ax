//! Recoil `m*.zmap` data structures.
use crate::static_assert_size;
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::Struct;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Struct)]
#[dotnet(val_struct)]
#[repr(C)]
pub struct MapColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
static_assert_size!(MapColor, 3);

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Struct)]
#[dotnet(val_struct)]
#[repr(C)]
pub struct MapVertex {
    pub x: f32,
    pub z: f32,
    pub y: f32,
}
static_assert_size!(MapVertex, 12);

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct MapFeature {
    pub color: MapColor,
    pub vertices: Vec<MapVertex>,
    pub objective: i32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(partial, namespace = "Mech3DotNet.Zbd")]
pub struct Zmap {
    pub unk04: u32,
    pub max_x: f32,
    pub max_y: f32,
    pub features: Vec<MapFeature>,
}
