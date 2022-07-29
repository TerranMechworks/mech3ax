#![warn(clippy::all, clippy::cargo)]
#![allow(clippy::identity_op)]
mod textures;

use ::serde::{Deserialize, Serialize};
use mech3ax_common::serde::base64;
use num_derive::FromPrimitive;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum TextureAlpha {
    None,
    Simple,
    Full,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, FromPrimitive, Copy, Clone)]
#[repr(u16)]
pub enum TextureStretch {
    None = 0,
    Vertical = 1,
    Horizontal = 2,
    Both = 3,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(u16)]
pub enum TexturePalette {
    None,
    Local(#[serde(with = "base64")] Vec<u8>),
    Global(i32, u16),
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
pub struct Manifest {
    pub texture_infos: Vec<TextureInfo>,
    pub global_palettes: Vec<Vec<u8>>,
}

pub use textures::{read_textures, write_textures};
