mod mw;
mod ng;
mod rc;

use crate::static_assert_size;
use crate::{Color, Vec3};
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::{RefStruct, ValStruct};

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Clone, Copy, ValStruct)]
#[repr(C)]
pub struct UvCoord {
    pub u: f32,
    pub v: f32,
}
static_assert_size!(UvCoord, 8);

#[derive(Debug, Serialize, Deserialize, RefStruct)]
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
