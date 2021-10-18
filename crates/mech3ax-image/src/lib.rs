#![warn(clippy::all, clippy::cargo)]
#![allow(clippy::identity_op)]
mod image;
mod textures;

pub use textures::{read_textures, write_textures, Manifest, TextureAlpha};
