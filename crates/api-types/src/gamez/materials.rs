use crate::serde::pointer_zero;
use crate::Color;
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::{Enum, Struct, Union};
use mech3ax_types::PrimitiveEnum;

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct CycleData {
    pub textures: Vec<String>,
    pub unk00: bool,
    pub unk04: u32,
    pub unk12: f32,
    pub info_ptr: u32,
    pub data_ptr: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PrimitiveEnum, Enum)]
#[repr(u32)]
pub enum Soil {
    Default = 0,
    Water = 1,
    Seafloor = 2,
    Quicksand = 3,
    Lava = 4,
    Fire = 5,
    Dirt = 6,
    Mud = 7,
    Grass = 8,
    Concrete = 9,
    Snow = 10,
    Mech = 11,
    Silt = 12,
    NoSlip = 13,
}

impl Soil {
    #[inline]
    pub const fn is_default(&self) -> bool {
        matches!(self, Self::Default)
    }
}

impl Default for Soil {
    #[inline]
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct TexturedMaterial {
    pub texture: String,
    // the GameZ data doesn't use the pointer (it stores the texture name index)
    #[serde(skip_serializing_if = "pointer_zero", default)]
    pub pointer: u32,
    // the Mechlib data doesn't have cycled textures
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub cycle: Option<CycleData>,
    #[serde(skip_serializing_if = "Soil::is_default", default)]
    pub soil: Soil,
    pub flag: bool,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct ColoredMaterial {
    pub color: Color,
    pub alpha: u8,
    #[serde(skip_serializing_if = "Soil::is_default", default)]
    pub soil: Soil,
}

#[derive(Debug, Serialize, Deserialize, Union)]
pub enum Material {
    Textured(TexturedMaterial),
    Colored(ColoredMaterial),
}
