#![warn(clippy::all, clippy::cargo)]
pub mod anim;
pub mod archive;
mod common;
pub mod gamez;
pub mod image;
pub mod interp;
pub(crate) mod macros;
pub mod messages;
pub mod motion;
pub mod nodes;
pub mod saves;
mod serde;
pub mod zmap;

pub(crate) use crate::macros::api::api;
pub(crate) use crate::macros::bit::bit;
pub(crate) use crate::macros::num::num;
pub(crate) use crate::macros::sum::sum;

pub use crate::serde::bytes::Bytes;
pub use common::{AffineMatrix, Color, Matrix, Quaternion, Range, Vec3};
