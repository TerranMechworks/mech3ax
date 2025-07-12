mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_types::{Ascii, Bool32, Maybe, Ptr, impl_as_bytes, primitive_enum};
pub(crate) use read::read_activ_prereqs;
pub(crate) use write::write_activ_prereqs;

primitive_enum! {
    /// Implicitly encoded in `ACTIVATION_PREREQUISITE`
    enum ActivPrereqType: u32 {
        /// `ANIMATION_LIST`
        Animation = 1,
        /// `OBJECT_ACTIVE_LIST` or `OBJECT_INACTIVE_LIST`
        Object = 2,
        /// `OBJECT_ACTIVE_LIST` or `OBJECT_INACTIVE_LIST`
        Parent = 3,
    }
}

/// ACTIVATION_PREREQUISITE
#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ActivPrereqHeaderC {
    /// `REQUIRED` or `OPTIONS`
    opt: Bool32, // 00
    /// `ANIMATION_LIST`, `OBJECT_ACTIVE_LIST`, or `OBJECT_INACTIVE_LIST`
    ty: Maybe<u32, ActivPrereqType>, // 04
}
impl_as_bytes!(ActivPrereqHeaderC, 8);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ActivPrereqAnimC {
    name: Ascii<32>, // 00
    zero32: u32,     // 32
    zero36: u32,     // 36
}
impl_as_bytes!(ActivPrereqAnimC, 40);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ActivPrereqObjC {
    active: u32,     // 00
    name: Ascii<32>, // 32
    ptr: Ptr,        // 36
}
impl_as_bytes!(ActivPrereqObjC, 40);
