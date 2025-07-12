mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::AnimMission;
use mech3ax_types::{Hex, impl_as_bytes};
pub use read::read_anim;
pub use write::write_anim;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct AnimHeaderC {
    signature: Hex<u32>, // 00
    version: u32,        // 04
}
impl_as_bytes!(AnimHeaderC, 8);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct AnimInfoC {
    zero00: u32,    // 000
    zero04: u32,    // 004
    zero08: u16,    // 008
    def_count: u16, // 010
    defs_ptr: u32,  // 012
    msg_count: u32, // 016
    msgs_ptr: u32,  // 020
    world_ptr: u32, // 024
    gravity: f32,   // 028
    zero32: u32,    // 032
    zero36: u32,    // 036
    zero40: u32,    // 040
    zero44: u32,    // 044
    zero48: u32,    // 048
    zero52: u32,    // 052
    zero56: u32,    // 056
    one60: u32,     // 060
    zero64: u32,    // 064
}
impl_as_bytes!(AnimInfoC, 68);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mission {
    C1V10,
    C1V12,
    C2V10,
    C2V12,
    C2V12De,
    C3V10,
    C3V12,
    C3V12De,
    C3V12DeP,
    C4V10,
    C4V12De,
    C4bV10,
    C4bV12De,
    T1V10,
    T1V12De,
    Unk,
}

impl Mission {
    pub(crate) fn from_api(api: AnimMission) -> Self {
        match api {
            AnimMission::MwC1V10 => Self::C1V10,
            AnimMission::MwC1V12 => Self::C1V12,
            AnimMission::MwC2V10 => Self::C2V10,
            AnimMission::MwC2V12 => Self::C2V12,
            AnimMission::MwC2V12De => Self::C2V12De,
            AnimMission::MwC3V10 => Self::C3V10,
            AnimMission::MwC3V12 => Self::C3V12,
            AnimMission::MwC3V12De => Self::C3V12De,
            AnimMission::MwC3V12DeP => Self::C3V12DeP,
            AnimMission::MwC4V10 => Self::C4V10,
            AnimMission::MwC4V12De => Self::C4V12De,
            AnimMission::MwC4bV10 => Self::C4bV10,
            AnimMission::MwC4bV12De => Self::C4bV12De,
            AnimMission::MwT1V10 => Self::T1V10,
            AnimMission::MwT1V12De => Self::T1V12De,
            AnimMission::Unknown => Self::Unk,
            _ => {
                log::warn!("Invalid mission {:?} for PM", api);
                Self::Unk
            }
        }
    }

    pub(crate) fn to_api(self) -> AnimMission {
        match self {
            Self::C1V10 => AnimMission::MwC1V10,
            Self::C1V12 => AnimMission::MwC1V12,
            Self::C2V10 => AnimMission::MwC2V10,
            Self::C2V12 => AnimMission::MwC2V12,
            Self::C2V12De => AnimMission::MwC2V12De,
            Self::C3V10 => AnimMission::MwC3V10,
            Self::C3V12 => AnimMission::MwC3V12,
            Self::C3V12De => AnimMission::MwC3V12De,
            Self::C3V12DeP => AnimMission::MwC3V12DeP,
            Self::C4V10 => AnimMission::MwC4V10,
            Self::C4V12De => AnimMission::MwC4V12De,
            Self::C4bV10 => AnimMission::MwC4bV10,
            Self::C4bV12De => AnimMission::MwC4bV12De,
            Self::T1V10 => AnimMission::MwT1V10,
            Self::T1V12De => AnimMission::MwT1V12De,
            Self::Unk => AnimMission::Unknown,
        }
    }

    pub(crate) fn from_defs_ptr(defs_ptr: u32) -> Self {
        match defs_ptr {
            0x03213F1C => Self::C1V10,
            0x03F5000C => Self::C1V12,
            0x032D17D4 => Self::C2V10,
            0x0360E598 => Self::C2V12,
            0x036086CC => Self::C2V12De,
            0x03A0ABE4 => Self::C3V10,
            0x03621248 => Self::C3V12,
            0x0361A508 => Self::C3V12De,
            0x035E8A24 => Self::C3V12DeP,
            0x032A8C00 => Self::C4V10,
            0x036705B4 => Self::C4V12De,
            0x0355000C => Self::C4bV10,
            0x037EB6BC => Self::C4bV12De,
            0x02FA6600 => Self::T1V10,
            0x030D9F94 => Self::T1V12De,
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
            Self::C1V10 => 0x03213F1C,
            Self::C1V12 => 0x03F5000C,
            Self::C2V10 => 0x032D17D4,
            Self::C2V12 => 0x0360E598,
            Self::C2V12De => 0x036086CC,
            Self::C3V10 => 0x03A0ABE4,
            Self::C3V12 => 0x03621248,
            Self::C3V12De => 0x0361A508,
            Self::C3V12DeP => 0x035E8A24,
            Self::C4V10 => 0x032A8C00,
            Self::C4V12De => 0x036705B4,
            Self::C4bV10 => 0x0355000C,
            Self::C4bV12De => 0x037EB6BC,
            Self::T1V10 => 0x02FA6600,
            Self::T1V12De => 0x030D9F94,
            Self::Unk => 0xDEADBEEF,
        }
    }

    pub(crate) fn world_ptr(self) -> u32 {
        match self {
            Self::C1V10 => 0x0284000C,
            Self::C1V12 => 0x02B4000C,
            Self::C2V10 => 0x0284000C,
            Self::C2V12 => 0x02B4000C,
            Self::C2V12De => 0x02B4000C,
            Self::C3V10 => 0x02F0002C,
            Self::C3V12 => 0x02B4000C,
            Self::C3V12De => 0x02B4000C,
            Self::C3V12DeP => 0x02B4000C,
            Self::C4V10 => 0x0284000C,
            Self::C4V12De => 0x02B4000C,
            Self::C4bV10 => 0x0284000C,
            Self::C4bV12De => 0x02B4000C,
            Self::T1V10 => 0x0284000C,
            Self::T1V12De => 0x02B4000C,
            Self::Unk => 0xDEADBEEF,
        }
    }
}
