#![warn(clippy::all, clippy::cargo)]
mod anim;
mod archive;
mod gamez;
mod image;
mod interp;
mod messages;
mod motion;
pub mod saves;
// TODO: make private once all external types (mainly saves) are in this crate
pub mod serde;
mod size;
mod types;

pub use anim::*;
pub use archive::*;
pub use gamez::*;
pub use image::*;
pub use interp::*;
pub use messages::*;
pub use motion::*;
pub use size::ReprSize;
pub use types::*;
