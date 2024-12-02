mod read;
mod write;
mod zero;

use crate::common::seq_def::SeqDefInfoC;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::AnimActivation;
use mech3ax_types::{bitflags, impl_as_bytes, Ascii, Maybe, Zeros};
pub(crate) use read::read_anim_def;
pub(crate) use write::write_anim_def;
pub(crate) use zero::{read_anim_def_zero, write_anim_def_zero};

bitflags! {
    pub(crate) struct AnimDefFlags: u32 {
        const EXECUTION_BY_RANGE = 1 << 1;
        const EXECUTION_BY_ZONE = 1 << 3;
        const HAS_CALLBACKS = 1 << 4;
        const RESET_TIME = 1 << 5;
        const NETWORK_LOG_SET = 1 << 10;
        const NETWORK_LOG_ON = 1 << 11;
        const SAVE_LOG_SET = 1 << 12;
        const SAVE_LOG_ON = 1 << 13;
        const AUTO_RESET_NODE_STATES = 1 << 16;
        // /// PM only
        // const LOCAL_NODES_ONLY = 1 << 19;
        const PROXIMITY_DAMAGE = 1 << 20;
    }
}

type Flags = Maybe<u32, AnimDefFlags>;
type Activ = Maybe<u8, AnimActivation>;

/// `ANIMATION_DEFINITION` in readers
#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Default)]
#[repr(C)]
struct AnimDefC {
    anim_name: Ascii<32>,            // 000
    name: Ascii<32>,                 // 032
    anim_ptr: u32,                   // 064
    anim_root_name: Ascii<32>,       // 068
    anim_root_ptr: u32,              // 100
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
    zero192: u32,                    // 192
    seq_defs_ptr: u32,               // 196
    reset_state: SeqDefInfoC,        // 200
    seq_def_count: u8,               // 264
    object_count: u8,                // 265
    node_count: u8,                  // 266
    light_count: u8,                 // 267
    puffer_count: u8,                // 268
    dynamic_sound_count: u8,         // 269
    static_sound_count: u8,          // 270
    effect_count: u8,                // 271
    activ_prereq_count: u8,          // 272
    activ_prereq_min_to_satisfy: u8, // 273
    anim_ref_count: u8,              // 274
    zero275: u8,                     // 275
    objects_ptr: u32,                // 276
    nodes_ptr: u32,                  // 280
    lights_ptr: u32,                 // 284
    puffers_ptr: u32,                // 288
    dynamic_sounds_ptr: u32,         // 292
    static_sounds_ptr: u32,          // 296
    effects_ptr: u32,                // 300
    activ_prereqs_ptr: u32,          // 304
    anim_refs_ptr: u32,              // 308
    zero312: u32,                    // 312
}
impl_as_bytes!(AnimDefC, 316);
