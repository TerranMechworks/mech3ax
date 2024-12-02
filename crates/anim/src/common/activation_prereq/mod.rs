mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_types::{impl_as_bytes, primitive_enum, Ascii, Bool32, Maybe, Ptr};
pub(crate) use read::read_activ_prereqs;
pub(crate) use write::write_activ_prereqs;

primitive_enum! {
    enum ActivPrereqType: u32 {
        Animation = 1,
        Object = 2,
        Parent = 3,
    }
}

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ActivPrereqHeaderC {
    opt: Bool32,                     // 00
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
