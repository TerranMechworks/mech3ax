use crate::serde::base64;
use ::serde::{Deserialize, Serialize};
use num_derive::FromPrimitive;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum TextureAlpha {
    None,
    Simple,
    Full,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, FromPrimitive)]
#[repr(u16)]
pub enum TextureStretch {
    None = 0,
    Vertical = 1,
    Horizontal = 2,
    Both = 3,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocalPalette {
    #[serde(with = "base64")]
    pub data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalPalette {
    pub index: i32,
    pub count: u16,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(u16)]
pub enum TexturePalette {
    None,
    Local(LocalPalette),
    Global(GlobalPalette),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextureInfo {
    pub name: String,
    pub alpha: TextureAlpha,
    pub width: u16,
    pub height: u16,
    pub stretch: TextureStretch,
    pub image_loaded: bool,
    pub alpha_loaded: bool,
    pub palette_loaded: bool,
    pub palette: TexturePalette,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextureManifest {
    pub texture_infos: Vec<TextureInfo>,
    pub global_palettes: Vec<Vec<u8>>,
}
