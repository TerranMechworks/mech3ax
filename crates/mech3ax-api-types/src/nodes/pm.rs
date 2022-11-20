use super::{BoundingBox, NodeFlags, Transformation};
use crate::types::Range;
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::{RefStruct, Union};

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct LodPm {
    pub name: String,

    pub level: bool,
    pub range: Range,
    pub unk64: f32,
    pub unk72: f32,

    // pub flags: NodeFlags,
    // pub zone_id: u32,
    // pub area_partition: Option<AreaPartition>,
    pub parent: u32,
    pub children: Vec<u32>,
    pub data_ptr: u32,
    pub parent_array_ptr: u32,
    pub children_array_ptr: u32,
    pub unk164: BoundingBox,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct Object3dPm {
    pub name: String,
    pub transformation: Option<Transformation>,
    pub matrix_signs: u32,

    pub flags: NodeFlags,
    // pub zone_id: u32,
    // pub area_partition: Option<AreaPartition>,
    pub mesh_index: i32,
    pub parent: Option<u32>,
    pub children: Vec<u32>,

    pub data_ptr: u32,
    pub parent_array_ptr: u32,
    pub children_array_ptr: u32,
    pub unk044: u32,
    pub unk112: u32,
    pub unk116: BoundingBox,
    pub unk140: BoundingBox,
    pub unk164: BoundingBox,
}

#[derive(Debug, Serialize, Deserialize, Union)]
pub enum NodePm {
    Lod(LodPm),
    Object3d(Object3dPm),
}
