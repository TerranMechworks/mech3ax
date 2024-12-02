mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_types::{impl_as_bytes, primitive_enum, Ascii, Bytes, Maybe, Ptr};
pub(crate) use read::{
    read_anim_refs, read_dynamic_sounds, read_effects, read_lights, read_nodes, read_objects,
    read_puffers, read_static_sounds,
};
pub(crate) use write::{
    write_anim_refs, write_dynamic_sounds, write_effects, write_lights, write_nodes, write_objects,
    write_puffers, write_static_sounds,
};

// NOT PM!
#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Default)]
#[repr(C)]
struct ObjectRefC {
    name: Ascii<32>, // 00
    zero32: u32,     // 32
    unk: Bytes<60>,  // 36
}
impl_as_bytes!(ObjectRefC, 96);

// NOT PM!
#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Default)]
#[repr(C)]
struct NodeRefC {
    name: Ascii<32>, // 00
    zero32: u32,     // 32
    ptr: Ptr,        // 36
}
impl_as_bytes!(NodeRefC, 40);

/// Fixup for one malformed node ref in RC.
const ABORT_TEST_RAW: Ascii<32> = Ascii::new(b"abort_test\0ng\0ame\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0");
/// Fixup for one malformed node ref in RC.
const ABORT_TEST_STR: &str = "abort_test";

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Default)]
#[repr(C)]
struct LightRefC {
    name: Ascii<32>, // 00
    flags: u32,      // 32
    ptr: Ptr,        // 36
    in_world: u32,   // 40
}
impl_as_bytes!(LightRefC, 44);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Default)]
#[repr(C)]
struct PufferRefC {
    name: Ascii<32>, // 00
    flags: u32,      // 32
    ptr: Ptr,        // 36
    in_world: u32,   // 40
}
impl_as_bytes!(PufferRefC, 44);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Default)]
#[repr(C)]
struct DynamicSoundRefC {
    name: Ascii<32>, // 00
    flags: u32,      // 32
    ptr: Ptr,        // 36
    in_world: u32,   // 40
}
impl_as_bytes!(DynamicSoundRefC, 44);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Default)]
#[repr(C)]
struct StaticSoundRefC {
    name: Ascii<32>, // 00
    zero32: u32,     // 32
}
impl_as_bytes!(StaticSoundRefC, 36);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Default)]
#[repr(C)]
struct EffectRefC {
    name: Ascii<32>, // 00
    unk32: u32,      // 32
}
impl_as_bytes!(EffectRefC, 36);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct AnimRefC {
    name: Ascii<64>,                 // 00
    ref_ty: Maybe<u32, AnimRefType>, // 64
    ptr: Ptr,                        // 68
}
impl_as_bytes!(AnimRefC, 72);

primitive_enum! {
    enum AnimRefType: u32 {
        CallAnimation = 0,
        // RC only?
        CallObjectConnector = 1,
    }
}
