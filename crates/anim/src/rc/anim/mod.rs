mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::AnimMission;
use mech3ax_types::{impl_as_bytes, Hex};
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
    zero00: u32,    // 00
    zero04: u32,    // 04
    zero08: u16,    // 08
    def_count: u16, // 10
    defs_ptr: u32,  // 12
    msg_count: u32, // 16
    msgs_ptr: u32,  // 20
    world_ptr: u32, // 24
    gravity: f32,   // 28
    zero32: u32,    // 32
    zero36: u32,    // 36
    zero40: u32,    // 40
    zero44: u32,    // 44
    zero48: u32,    // 48
    zero52: u32,    // 52
    zero56: u32,    // 56
}
impl_as_bytes!(AnimInfoC, 60);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mission {
    M01,
    M02,
    M03,
    M04,
    M05,
    M06,
    M07,
    M08,
    M09,
    M10,
    M11,
    M12,
    M13,
    Unk,
}

impl Mission {
    pub(crate) fn from_api(api: AnimMission) -> Self {
        match api {
            AnimMission::RcM01 => Self::M01,
            AnimMission::RcM02 => Self::M02,
            AnimMission::RcM03 => Self::M03,
            AnimMission::RcM04 => Self::M04,
            AnimMission::RcM05 => Self::M05,
            AnimMission::RcM06 => Self::M06,
            AnimMission::RcM07 => Self::M07,
            AnimMission::RcM08 => Self::M08,
            AnimMission::RcM09 => Self::M09,
            AnimMission::RcM10 => Self::M10,
            AnimMission::RcM11 => Self::M11,
            AnimMission::RcM12 => Self::M12,
            AnimMission::RcM13 => Self::M13,
            AnimMission::Unknown => Self::Unk,
            _ => {
                log::warn!("Invalid mission {:?} for PM", api);
                Self::Unk
            }
        }
    }

    pub(crate) fn to_api(self) -> AnimMission {
        match self {
            Self::M01 => AnimMission::RcM01,
            Self::M02 => AnimMission::RcM02,
            Self::M03 => AnimMission::RcM03,
            Self::M04 => AnimMission::RcM04,
            Self::M05 => AnimMission::RcM05,
            Self::M06 => AnimMission::RcM06,
            Self::M07 => AnimMission::RcM07,
            Self::M08 => AnimMission::RcM08,
            Self::M09 => AnimMission::RcM09,
            Self::M10 => AnimMission::RcM10,
            Self::M11 => AnimMission::RcM11,
            Self::M12 => AnimMission::RcM12,
            Self::M13 => AnimMission::RcM13,
            Self::Unk => AnimMission::Unknown,
        }
    }
    pub(crate) fn from_defs_ptr(defs_ptr: u32) -> Self {
        match defs_ptr {
            0x02F1D378 => Self::M01,
            0x02F53FD8 => Self::M02,
            0x02F830F0 => Self::M03,
            0x02ECD990 => Self::M04,
            0x02EDD314 => Self::M05,
            0x034DEF64 => Self::M06,
            0x02D359D8 => Self::M07,
            0x02D4B4F4 => Self::M08,
            0x02D71A80 => Self::M09,
            0x02D7A860 => Self::M10,
            0x02CF1B74 => Self::M11,
            0x02D405E0 => Self::M12,
            0x02E0000C => Self::M13,
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
            Self::M01 => 0x02F1D378,
            Self::M02 => 0x02F53FD8,
            Self::M03 => 0x02F830F0,
            Self::M04 => 0x02ECD990,
            Self::M05 => 0x02EDD314,
            Self::M06 => 0x034DEF64,
            Self::M07 => 0x02D359D8,
            Self::M08 => 0x02D4B4F4,
            Self::M09 => 0x02D71A80,
            Self::M10 => 0x02D7A860,
            Self::M11 => 0x02CF1B74,
            Self::M12 => 0x02D405E0,
            Self::M13 => 0x02E0000C,
            Self::Unk => 0xDEADBEEF,
        }
    }

    pub(crate) fn world_ptr(self) -> u32 {
        0x0260000C
    }
}
