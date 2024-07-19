mod mw;
mod ng;
mod rc;

use crate::impl_as_bytes;
use crate::{Color, Vec3};
use ::serde::{Deserialize, Serialize};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_metadata_proc_macro::Struct;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    Serialize,
    Deserialize,
    NoUninit,
    AnyBitPattern,
    Struct,
)]
#[dotnet(val_struct)]
#[repr(C)]
pub struct UvCoord {
    pub u: f32,
    pub v: f32,
}
impl_as_bytes!(UvCoord, 8);

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct MeshLight {
    pub unk00: u32,
    pub unk04: u32,
    pub unk08: f32,
    pub extra: Vec<Vec3>,
    pub unk24: u32,
    pub color: Color,
    pub flags: u16,
    pub ptr: u32,
    pub unk48: f32,
    pub unk52: f32,
    pub unk56: f32,
    pub unk60: u32,
    pub unk64: f32,
    pub unk68: f32,
    pub unk72: f32,
}

pub use mw::*;
pub use ng::*;
pub use rc::*;
