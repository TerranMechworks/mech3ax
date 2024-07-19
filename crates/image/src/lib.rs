#![warn(clippy::all, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)] // image crate uses outdated bitflags
#![allow(clippy::identity_op)]
mod textures;

pub use textures::{read_textures, write_textures};
