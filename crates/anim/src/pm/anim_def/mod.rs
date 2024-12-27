mod read;
mod write;
mod zero;

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
#[derive(Debug, Clone, Copy, PartialEq, NoUninit, AnyBitPattern)]
#[repr(C)]
struct AnimDefC {
    anim_name: Ascii<32>,            // 000
    unknowns_ptr: Ptr,               // 032
    unknowns_count: Ptr,             // 036
    name: Ascii<32>,                 // 040
    anim_ptr: Ptr,                   // 072
    anim_root_name: Ascii<32>,       // 076
    anim_root_ptr: Ptr,              // 108
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
    seq_defs_ptr: Ptr,               // 204
    reset_state_ptr: Ptr,            // 208
    unknown_seq_ptr: Ptr,            // 212
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
    objects_ptr: Ptr,                // 228
    nodes_ptr: Ptr,                  // 232
    lights_ptr: Ptr,                 // 236
    puffers_ptr: Ptr,                // 240
    dynamic_sounds_ptr: Ptr,         // 244
    static_sounds_ptr: Ptr,          // 248
    effects_ptr: Ptr,                // 252
    activ_prereqs_ptr: Ptr,          // 256
    anim_refs_ptr: Ptr,              // 260
    zero264: u32,                    // 264
}
impl_as_bytes!(AnimDefC, 268);

impl Default for AnimDefC {
    fn default() -> Self {
        use bytemuck::Zeroable as _;
        Self {
            anim_ptr: Ptr::INVALID,
            anim_root_ptr: Ptr::INVALID,
            activation: AnimActivation::OnCall.maybe(),
            ..AnimDefC::zeroed()
        }
    }
}
