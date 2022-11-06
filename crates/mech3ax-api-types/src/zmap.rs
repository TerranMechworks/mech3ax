use crate::static_assert_size;
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::{RefStruct, ValStruct};

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Clone, Copy, ValStruct)]
#[repr(C)]
pub struct MapVertex {
    pub x: f32,
    pub z: f32,
    pub y: f32,
}
static_assert_size!(MapVertex, 12);

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct MapChunk {
    pub flag1: u8,
    pub flag2: u8,
    pub flag3: u8,
    pub vertices: Vec<MapVertex>,
    pub tail: i32,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct MapRc {
    pub unk04: u32,
    pub max_x: f32,
    pub max_y: f32,
    pub chunks: Vec<MapChunk>,
}
