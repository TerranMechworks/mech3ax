mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::AffineMatrix;
use mech3ax_types::{impl_as_bytes, primitive_enum, Ascii, Hex, Maybe, Ptr};
pub(crate) use read::{
    affine_to_bin, read_anim_refs, read_dynamic_sounds, read_effects, read_lights, read_nodes,
    read_objects, read_puffers, read_static_sounds,
};
pub(crate) use write::{
    bin_to_affine, write_anim_refs, write_dynamic_sounds, write_effects, write_lights, write_nodes,
    write_objects, write_puffers, write_static_sounds,
};

/// "Object" references are a list of "objects" (usually Object3D nodes) that
/// sequence definition events of an animation definition uses/references.
///
/// These are implicitly derived from the reader data. It's unclear to me what
/// determines if an argument is a node or an object, possibly the event itself.
///
/// PM uses a different structure!
#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Default)]
#[repr(C)]
struct ObjectRefC {
    name: Ascii<32>,        // 00
    zero32: u32,            // 32
    ptr: Ptr,               // 36
    flags: Hex<u32>,        // 40
    flags_merged: Hex<u32>, // 44,
    affine: AffineMatrix,   // 48
}
impl_as_bytes!(ObjectRefC, 96);

/// Node references are a list of nodes that sequence definition events of an
/// animation definition uses/references.
///
/// These are implicitly derived from the reader data.
///
/// PM uses a different structure!
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

/// Light references are a list of light nodes that sequence definition events
/// of an animation definition uses/references.
///
/// These are implicitly derived from the reader data.
#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Default)]
#[repr(C)]
struct LightRefC {
    name: Ascii<32>, // 00
    flags: u32,      // 32
    ptr: Ptr,        // 36
    in_world: u32,   // 40
}
impl_as_bytes!(LightRefC, 44);

/// Puffer references are a list of puffer nodes that sequence definition events
/// of an animation definition uses/references.
///
/// These are implicitly derived from the reader data.
#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Default)]
#[repr(C)]
struct PufferRefC {
    name: Ascii<32>, // 00
    flags: u32,      // 32
    ptr: Ptr,        // 36
    in_world: u32,   // 40
}
impl_as_bytes!(PufferRefC, 44);

/// Dynamic sound references are a list of sound nodes that sequence definition
/// events of an animation definition uses/references.
///
/// These are implicitly derived from the reader data.
#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Default)]
#[repr(C)]
struct DynamicSoundRefC {
    name: Ascii<32>, // 00
    flags: u32,      // 32
    ptr: Ptr,        // 36
    in_world: u32,   // 40
}
impl_as_bytes!(DynamicSoundRefC, 44);

/// Static sound references are a list of sounds (not sound nodes!) that
/// sequence definition events of an animation definition uses/references.
///
/// These are implicitly derived from the reader data.
#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Default)]
#[repr(C)]
struct StaticSoundRefC {
    name: Ascii<32>, // 00
    zero32: u32,     // 32
}
impl_as_bytes!(StaticSoundRefC, 36);

/// Effect references are a list of effects that sequence definition events of
/// an animation definition uses/references.
///
/// These are implicitly derived from the reader data.
///
/// Effects seem to basically be cycled textures. Note that although technically
/// MW and PM still support effects, all textures support cycling and this has
/// become obsolete.
#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Default)]
#[repr(C)]
struct EffectRefC {
    name: Ascii<32>, // 00
    unk32: u32,      // 32
}
impl_as_bytes!(EffectRefC, 36);

/// Anim references are a list of animation definitions that sequence definition
/// events of an animation definition uses/references.
///
/// These are implicitly derived from the reader data.
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
        /// `CALL_ANIMATION`
        CallAnimation = 0,
        // `CALL_OBJECT_CONNECTOR`? RC only?
        CallObjectConnector = 1,
    }
}
