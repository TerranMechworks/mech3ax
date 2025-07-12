mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::AnimMission;
use mech3ax_types::{Bool32, Hex, impl_as_bytes};
pub use read::read_anim;
pub use write::write_anim;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct AnimHeaderC {
    signature: Hex<u32>, // 00
    version: u32,        // 04
    timestamp: u32,      // 08
}
impl_as_bytes!(AnimHeaderC, 12);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct AnimInfoC {
    zero00: u32,       // 000
    zero04: u32,       // 004
    zero08: u16,       // 008
    def_count: u16,    // 010
    defs_ptr: u32,     // 012
    script_count: u32, // 016
    scripts_ptr: u32,  // 020
    msg_count: u32,    // 024
    msgs_ptr: u32,     // 028
    world_ptr: u32,    // 032
    gravity: f32,      // 036
    unk40: u32,        // 040
    zero44: u32,       // 044
    zero48: u32,       // 048
    zero52: u32,       // 052
    zero56: u32,       // 056
    one60: u32,        // 060
    zero64: u32,       // 064
    zero68: u32,       // 068
    zero72: u32,       // 072
    zero76: u32,       // 076
    zero80: u32,       // 080
    zero84: u32,       // 084
    zero88: u32,       // 088
    zero92: u32,       // 092
    zero96: u32,       // 096
    zero100: u32,      // 100
    zero104: u32,      // 104
}
impl_as_bytes!(AnimInfoC, 108);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct SiScriptC {
    script_name_ptr: u32,  // 00
    object_name_ptr: u32,  // 04
    script_name_len: u8,   // 08
    object_name_len: u8,   // 09
    pad10: u16,            // 10
    spline_interp: Bool32, // 12
    frame_count: u32,      // 16
    script_data_len: u32,  // 20
    script_data_ptr: u32,  // 24
}
impl_as_bytes!(SiScriptC, 28);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mission {
    C1,
    C2,
    C3,
    C4,
    Unk,
}

impl Mission {
    pub(crate) fn from_api(api: AnimMission) -> Self {
        match api {
            AnimMission::PmC1 => Self::C1,
            AnimMission::PmC2 => Self::C2,
            AnimMission::PmC3 => Self::C3,
            AnimMission::PmC4 => Self::C4,
            AnimMission::Unknown => Self::Unk,
            _ => {
                log::warn!("Invalid mission {:?} for PM", api);
                Self::Unk
            }
        }
    }

    pub(crate) fn to_api(self) -> AnimMission {
        match self {
            Self::C1 => AnimMission::PmC1,
            Self::C2 => AnimMission::PmC2,
            Self::C3 => AnimMission::PmC3,
            Self::C4 => AnimMission::PmC4,
            Self::Unk => AnimMission::Unknown,
        }
    }

    pub(crate) fn from_defs_ptr(defs_ptr: u32) -> Self {
        match defs_ptr {
            0x12D3AFF8 => Self::C1,
            0x11221FF8 => Self::C2,
            0x0C1CEF10 => Self::C3,
            0x12245FF8 => Self::C4,
            0xDEADBEEF => Self::Unk,
            _ => {
                log::warn!(
                    "Unknown defs pointer 0x{:08X}, ZBD translation may be inaccurate",
                    defs_ptr
                );
                Self::Unk
            }
        }
    }

    pub(crate) fn defs_ptr(self) -> u32 {
        match self {
            Self::C1 => 0x12D3AFF8,
            Self::C2 => 0x11221FF8,
            Self::C3 => 0x0C1CEF10,
            Self::C4 => 0x12245FF8,
            Self::Unk => 0xDEADBEEF,
        }
    }

    pub(crate) fn scripts_ptr(self) -> u32 {
        match self {
            Self::C1 => 0x06A6A008,
            Self::C2 => 0x0A5BF008,
            Self::C3 => 0x0618C7A8,
            Self::C4 => 0x06531008,
            Self::Unk => 0xDEADBEEF,
        }
    }

    pub(crate) fn world_ptr(self) -> u32 {
        0x05320020
    }

    pub(crate) fn unk40(self) -> u32 {
        match self {
            Self::C1 => 1,
            Self::C2 => 0,
            Self::C3 => 0,
            Self::C4 => 0,
            Self::Unk => 0,
        }
    }
}
