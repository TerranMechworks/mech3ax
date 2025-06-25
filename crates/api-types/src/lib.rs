#![warn(clippy::all, clippy::cargo)]
pub mod anim;
pub mod archive;
mod common;
pub(crate) mod flags;
pub mod gamez;
pub mod image;
pub mod interp;
pub mod messages;
pub mod motion;
pub mod nodes;
pub mod saves;
mod serde;
pub mod zmap;

pub(crate) use crate::flags::flags;
pub use crate::serde::bytes::Bytes;
pub use common::{AffineMatrix, Color, Matrix, Quaternion, Range, Vec3};
