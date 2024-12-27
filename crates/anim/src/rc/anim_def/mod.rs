mod read;
mod write;
mod zero;

use crate::common::seq_def::SeqDefInfoC;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::AnimActivation;
use mech3ax_types::{bitflags, impl_as_bytes, Ascii, Maybe, Ptr, Zeros};
pub(crate) use read::read_anim_def;
pub(crate) use write::write_anim_def;
pub(crate) use zero::{read_anim_def_zero, write_anim_def_zero};

bitflags! {
    struct AnimDefFlags: u32 {
        const EXECUTION_BY_RANGE = 1 << 1;
        const EXECUTION_BY_ZONE = 1 << 3;
        const HAS_CALLBACKS = 1 << 4;
        const RESET_TIME = 1 << 5;
        const NETWORK_LOG = 1 << 10;
        // /// MW/PM only
        // const NETWORK_LOG_ON = 1 << 11;
        const SAVE_LOG = 1 << 12;
        // /// MW/PM only
        // const SAVE_LOG_ON = 1 << 13;
        // /// MW/PM only
        // const AUTO_RESET_NODE_STATES = 1 << 16;
        // /// PM only
        // const LOCAL_NODES_ONLY = 1 << 19;
        // /// MW/PM only
        // const PROXIMITY_DAMAGE = 1 << 20;
    }
}

type Flags = Maybe<u32, AnimDefFlags>;
type Activ = Maybe<u8, AnimActivation>;

/// `ANIMATION_DEFINITION` in readers
#[derive(Debug, Clone, Copy, PartialEq, NoUninit, AnyBitPattern)]
#[repr(C)]
struct AnimDefC {
    anim_name: Ascii<32>,            // 000
    name: Ascii<32>,                 // 032
    anim_ptr: Ptr,                   // 064
    anim_root_name: Ascii<32>,       // 068
    anim_root_ptr: Ptr,              // 100
    zero104: Zeros<44>,              // 104
    flags: Flags,                    // 148
    status: u8,                      // 152
    activation: Activ,               // 153
    execution_priority: u8,          // 154
    two155: u8,                      // 155
    exec_by_range_min: f32,          // 156
    exec_by_range_max: f32,          // 160
    reset_time: f32,                 // 164
    zero168: f32,                    // 168
    max_health: f32,                 // 172
    cur_health: f32,                 // 176
    zero180: u32,                    // 180
    zero184: u32,                    // 184
    zero188: u32,                    // 188
    seq_defs_ptr: Ptr,               // 192
    reset_state: SeqDefInfoC,        // 196
    seq_def_count: u8,               // 260
    object_count: u8,                // 261
    node_count: u8,                  // 262
    light_count: u8,                 // 263
    dynamic_sound_count: u8,         // 264
    static_sound_count: u8,          // 265
    effect_count: u8,                // 266
    activ_prereq_count: u8,          // 267
    activ_prereq_min_to_satisfy: u8, // 268
    anim_ref_count: u8,              // 269
    zero270: u8,                     // 270
    zero271: u8,                     // 271
    objects_ptr: Ptr,                // 272
    nodes_ptr: Ptr,                  // 276
    lights_ptr: Ptr,                 // 280
    dynamic_sounds_ptr: Ptr,         // 284
    static_sounds_ptr: Ptr,          // 288
    effects_ptr: Ptr,                // 292
    activ_prereqs_ptr: Ptr,          // 296
    anim_refs_ptr: Ptr,              // 300
    zero304: u32,                    // 304
}
impl_as_bytes!(AnimDefC, 308);

impl Default for AnimDefC {
    fn default() -> Self {
        use bytemuck::Zeroable as _;
        Self {
            activation: AnimActivation::OnCall.maybe(),
            ..AnimDefC::zeroed()
        }
    }
}

/// These anim def seq defs ptrs indicate the reset time is 0.0, and not -1.0
const RESET_TIME_BORK: &[u32] = &[
    0x016FACB0, 0x016FCC50, 0x0170FB10, 0x01726BA0, 0x0172FD60, 0x01734560, 0x0173E850, 0x017495E0,
    0x01751B80, 0x017AF4F0, 0x017B1EE0, 0x017F5BA0, 0x01979E50, 0x01A0A7E0, 0x01A7E3F0, 0x01AB1860,
    0x02E58900, 0x032067D0, 0x032663F0,
];
