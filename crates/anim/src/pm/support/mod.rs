mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::AffineMatrix;
use mech3ax_types::{impl_as_bytes, Ascii, Hex, Ptr};
pub(crate) use read::{read_nodes, read_objects};
pub(crate) use write::{write_nodes, write_objects};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ObjectRefC {
    name: Ascii<32>,      // 00
    zero32: u32,          // 32
    ptr: Ptr,             // 36
    flags: Hex<u16>,      // 40
    root_idx: u16,        // 42,
    affine: AffineMatrix, // 44
}
impl_as_bytes!(ObjectRefC, 92);

impl Default for ObjectRefC {
    #[inline]
    fn default() -> Self {
        Self {
            name: Ascii::default(),
            zero32: 0,
            ptr: Ptr::INVALID,
            flags: Hex(0),
            root_idx: 0,
            affine: AffineMatrix::ZERO,
        }
    }
}

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct NodeRefC {
    flags: u16,      // 00
    root_idx: u16,   // 02,
    name: Ascii<32>, // 04
    zero36: u32,     // 36
    ptr: Ptr,        // 40
}
impl_as_bytes!(NodeRefC, 44);

impl Default for NodeRefC {
    #[inline]
    fn default() -> Self {
        Self {
            flags: 0,
            root_idx: 0,
            name: Ascii::default(),
            zero36: 0,
            ptr: Ptr::INVALID,
        }
    }
}
