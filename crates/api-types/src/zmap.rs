//! Recoil `m*.zmap` data structures.
use crate::Vec3;
use ::serde::{Deserialize, Serialize};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_metadata_proc_macro::Struct;
use mech3ax_types::impl_as_bytes;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, NoUninit, AnyBitPattern, Struct,
)]
#[dotnet(val_struct)]
#[repr(C)]
pub struct MapColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
impl_as_bytes!(MapColor, 3);

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct MapFeature {
    pub color: MapColor,
    pub vertices: Vec<Vec3>,
    pub objective: i32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(partial, namespace = "Mech3DotNet.Zbd")]
pub struct Zmap {
    pub unk04: u32,
    pub min: Vec3,
    pub max: Vec3,
    pub features: Vec<MapFeature>,
}
