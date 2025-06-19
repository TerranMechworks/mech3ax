use crate::serde::pointer_zero;
use crate::Color;
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::{Enum, Struct, Union};
use mech3ax_types::primitive_enum;

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct CycleData {
    pub textures: Vec<String>,
    pub looping: bool,
    pub speed: f32,

    pub current_frame: i32,
    pub cycle_ptr: u32,
    pub tex_map_ptr: u32,
}

primitive_enum! {
    #[derive(Serialize, Deserialize, Enum)]
    pub enum Soil: u32 {
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

impl Material {
    #[inline]
    pub fn is_cycled(&self) -> bool {
        match self {
            Self::Colored(_) => false,
            Self::Textured(textured) => textured.cycle.is_some(),
        }
    }
}
