#![warn(clippy::all, clippy::cargo)]
#![allow(clippy::identity_op)]
mod flags;
mod math;
pub mod mw;
pub mod pm;
mod range;
pub mod types;

pub use types::NodeVariantMw;
