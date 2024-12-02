mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_types::{impl_as_bytes, Ascii, Bytes, Ptr};
pub(crate) use read::{read_nodes, read_objects};
pub(crate) use write::{write_nodes, write_objects};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ObjectRefC {
    name: Ascii<32>, // 00
    zero32: u32,     // 32
    mone36: u32,     // 36
    unk: Bytes<52>,  // 40
}
impl_as_bytes!(ObjectRefC, 92);

impl Default for ObjectRefC {
    #[inline]
    fn default() -> Self {
        Self {
            name: Ascii::default(),
            zero32: 0,
            mone36: u32::MAX,
            unk: Bytes::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct NodeRefC {
    zero00: u32,     // 00
    name: Ascii<32>, // 04
    zero36: u32,     // 36
    ptr: Ptr,        // 40
}
impl_as_bytes!(NodeRefC, 44);

impl Default for NodeRefC {
    #[inline]
    fn default() -> Self {
        Self {
            zero00: 0,
            name: Ascii::default(),
            zero36: 0,
            ptr: Ptr::INVALID,
        }
    }
}
