use crate::serde::pointer_zero;
use crate::types::Color;
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::{RefStruct, Union};

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct CycleData {
    pub textures: Vec<String>,
    pub unk00: bool,
    pub unk04: u32,
    pub unk12: f32,
    pub info_ptr: u32,
    pub data_ptr: u32,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct TexturedMaterial {
    pub texture: String,
    // the GameZ data doesn't use the pointer (it stores the texture name index)
    #[serde(skip_serializing_if = "pointer_zero", default)]
    pub pointer: u32,
    // the Mechlib data doesn't have cycled textures
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub cycle: Option<CycleData>,
    pub specular: f32,
    pub flag: bool,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct ColoredMaterial {
    pub color: Color,
    pub alpha: u8,
    pub specular: f32,
}

#[derive(Debug, Serialize, Deserialize, Union)]
pub enum Material {
    Textured(TexturedMaterial),
    Colored(ColoredMaterial),
}
