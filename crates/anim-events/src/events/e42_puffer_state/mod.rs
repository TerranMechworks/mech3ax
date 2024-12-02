mod mw;
mod pm;
mod read;
mod write;

use super::{EventMw, EventPm};
use crate::types::Idx32;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::PufferStateColor;
use mech3ax_api_types::{Range, Vec3};
use mech3ax_types::{bitflags, impl_as_bytes, Ascii, Bool32, Maybe};

bitflags! {
    struct PufferStateFlags: u32 {
        const TRANSLATE_ABS = 1 << 0;       // 0x000001
        const AT_NODE = 1 << 1;             // 0x000002
        const ACTIVE_STATE = 1 << 2;        // 0x000004
        const LOCAL_VELOCITY = 1 << 3;      // 0x000008
        const WORLD_VELOCITY = 1 << 4;      // 0x000010
        const MIN_RANDOM_VELOCITY = 1 << 5; // 0x000020
        const MAX_RANDOM_VELOCITY = 1 << 6; // 0x000040
        const INTERVAL_TYPE = 1 << 7;       // 0x000080
        const INTERVAL_VALUE = 1 << 8;      // 0x000100
        const SIZE_RANGE = 1 << 9;          // 0x000200
        const LIFETIME_RANGE = 1 << 10;     // 0x000400
        const DEVIATION_DISTANCE = 1 << 11; // 0x000800
        const FADE_RANGE = 1 << 12;         // 0x001000
        const GROWTH_FACTORS = 1 << 13;     // 0x002000
        const TEXTURES = 1 << 14;           // 0x004000
        const START_AGE_RANGE = 1 << 15;    // 0x008000
        const WORLD_ACCELERATION = 1 << 16; // 0x010000
        const FRICTION = 1 << 17;           // 0x020000
        const COLORS = 1 << 18;             // 0x040000
        const UNKNOWN_RANGE = 1 << 19;      // 0x080000
        const WIND_FACTOR = 1 << 20;        // 0x100000
        const NUMBER = 1 << 21;             // 0x200000
        const PRIORITY = 1 << 22;           // 0x400000
    }
}

type Flags = Maybe<u32, PufferStateFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct PufferStateCommon {
    puffer_name: Ascii<32>,    // 000
    puffer_index: Idx32,       // 032
    flags: Flags,              // 036
    node_index: Idx32,         // 040
    active_state: u32,         // 044
    translate: Vec3,           // 048
    local_velocity: Vec3,      // 060
    world_velocity: Vec3,      // 072
    min_random_velocity: Vec3, // 084
    max_random_velocity: Vec3, // 096
    world_acceleration: Vec3,  // 108
    interval_type: Bool32,     // 120
    interval_value: f32,       // 124
    size_range: Range,         // 128
    lifetime_range: Range,     // 136
    start_age_range: Range,    // 144
    deviation_distance: f32,   // 152
    unk_range: Range,          // 156
    fade_range: Range,         // 164
    friction: f32,             // 172
    wind_factor: f32,          // 176
    priority: f32,             // 180
}
impl_as_bytes!(PufferStateCommon, 184);

#[derive(Debug, Clone, Copy, PartialEq, NoUninit, AnyBitPattern)]
#[repr(C)]
struct PufferStateTextureC {
    run_time: f32,   // 032
    name: Ascii<32>, // 000
}
impl_as_bytes!(PufferStateTextureC, 36);

impl PufferStateTextureC {
    pub const ZERO: Self = Self {
        run_time: 0.0,
        name: Ascii::zero(),
    };
}

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct PufferStateTextures {
    has_run_time: Bool32,           // 000
    texture_0: PufferStateTextureC, // 004
    texture_1: PufferStateTextureC, // 040
    texture_2: PufferStateTextureC, // 076
    texture_3: PufferStateTextureC, // 112
    texture_4: PufferStateTextureC, // 148
    texture_5: PufferStateTextureC, // 184
}
impl_as_bytes!(PufferStateTextures, 220);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct PufferStateColors {
    count: u32,                // 000
    color_0: PufferStateColor, // 004
    color_1: PufferStateColor, // 024
    color_2: PufferStateColor, // 044
    color_3: PufferStateColor, // 064
    color_4: PufferStateColor, // 084
    color_5: PufferStateColor, // 104
}
impl_as_bytes!(PufferStateColors, 124);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct PufferStateGrowths {
    count: u32,      // 00
    growth_0: Range, // 04
    growth_1: Range, // 12
    growth_2: Range, // 20
    growth_3: Range, // 28
    growth_4: Range, // 36
    growth_5: Range, // 44
}
impl_as_bytes!(PufferStateGrowths, 52);
