use crate::serde::bytes;
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::{Enum, RefStruct, Union};
use num_derive::FromPrimitive;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Enum)]
pub enum TextureAlpha {
    None,
    Simple,
    Full,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, FromPrimitive, Enum)]
#[repr(u16)]
pub enum TextureStretch {
    None = 0,
    Vertical = 1,
    Horizontal = 2,
    Both = 3,
    /// Crimson Skies only
    Unk4 = 4,
    /// Crimson Skies only
    Unk7 = 7,
    /// Crimson Skies only
    Unk8 = 8,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct PaletteData {
    #[serde(with = "bytes")]
    pub data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct GlobalPalette {
    pub index: i32,
    pub count: u16,
}

#[derive(Debug, Serialize, Deserialize, Union)]
#[repr(u16)]
pub enum TexturePalette {
    None,
    Local(PaletteData),
    Global(GlobalPalette),
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct TextureInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub rename: Option<String>,
    pub alpha: TextureAlpha,
    pub width: u16,
    pub height: u16,
    pub stretch: TextureStretch,
    pub image_loaded: bool,
    pub alpha_loaded: bool,
    pub palette_loaded: bool,
    pub palette: TexturePalette,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct TextureManifest {
    pub texture_infos: Vec<TextureInfo>,
    pub global_palettes: Vec<PaletteData>,
}
