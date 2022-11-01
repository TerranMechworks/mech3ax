#![warn(clippy::all, clippy::cargo)]
mod anim;
mod archive;
mod gamez;
mod image;
mod interp;
mod messages;
mod motion;
pub mod saves;
mod serde;
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

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Hide<T>(pub T);

impl<T> std::fmt::Debug for Hide<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("...")
    }
}

impl<T> From<T> for Hide<T> {
    fn from(inner: T) -> Self {
        Self(inner)
    }
}
