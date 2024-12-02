mod read;
mod write;
mod zero;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::AnimActivation;
use mech3ax_types::{bitflags, impl_as_bytes, Ascii, Maybe, Zeros};
pub(crate) use read::read_anim_def;
pub(crate) use write::write_anim_def;
pub(crate) use zero::{read_anim_def_zero, write_anim_def_zero};

bitflags! {
    struct AnimDefFlags: u32 {
        const EXECUTION_BY_RANGE = 1 << 1;
        const EXECUTION_BY_ZONE = 1 << 3;
        const HAS_CALLBACKS = 1 << 4;
        const RESET_TIME = 1 << 5;
        const NETWORK_LOG_SET = 1 << 10;
        const NETWORK_LOG_ON = 1 << 11;
        const SAVE_LOG_SET = 1 << 12;
        const SAVE_LOG_ON = 1 << 13;
        const AUTO_RESET_NODE_STATES = 1 << 16;
        // PM only
        const LOCAL_NODES_ONLY = 1 << 19;
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
    unknowns_ptr: u32,               // 032
    unknowns_count: u32,             // 036
    name: Ascii<32>,                 // 040
    anim_ptr: u32,                   // 072
    anim_root_name: Ascii<32>,       // 076
    anim_root_ptr: u32,              // 108
    zero112: Zeros<44>,              // 112
    flags: Flags,                    // 156
    status: u8,                      // 160
    activation: Activ,               // 161
    execution_priority: u8,          // 162
    two163: u8,                      // 163
    exec_by_range_min: f32,          // 164
    exec_by_range_max: f32,          // 168
    reset_time: f32,                 // 172
    zero176: f32,                    // 176
    max_health: f32,                 // 180
    cur_health: f32,                 // 184
    zero188: u32,                    // 188
    zero192: u32,                    // 192
    zero196: u32,                    // 196
    zero200: u32,                    // 200
    seq_defs_ptr: u32,               // 204
    reset_state_ptr: u32,            // 208
    unknown_seq_ptr: u32,            // 212
    seq_def_count: u8,               // 216
    object_count: u8,                // 217
    node_count: u8,                  // 218
    light_count: u8,                 // 219
    puffer_count: u8,                // 220
    dynamic_sound_count: u8,         // 221
    static_sound_count: u8,          // 222
    effect_count: u8,                // 223
    activ_prereq_count: u8,          // 224
    activ_prereq_min_to_satisfy: u8, // 225
    anim_ref_count: u8,              // 226
    zero227: u8,                     // 227
    objects_ptr: u32,                // 228
    nodes_ptr: u32,                  // 232
    lights_ptr: u32,                 // 236
    puffers_ptr: u32,                // 240
    dynamic_sounds_ptr: u32,         // 244
    static_sounds_ptr: u32,          // 248
    effects_ptr: u32,                // 252
    activ_prereqs_ptr: u32,          // 256
    anim_refs_ptr: u32,              // 260
    zero264: u32,                    // 264
}
impl_as_bytes!(AnimDefC, 268);
