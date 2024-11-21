use super::utils::assert_color;
use super::ScriptObject;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::{FogState, FogType};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::{impl_as_bytes, AsBytes as _, Color, Range};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::string::str_to_c_padded;
use mech3ax_common::{assert_that, Result};
use mech3ax_types::Ascii;
use std::io::{Read, Write};

const DEFAULT_FOG_NAME: &str = "default_fog_name";

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FogStateFlags: u32 {
        // const STATE = 1 << 0;
        const COLOR = 1 << 1;
        const ALTITUDE = 1 << 2;
        const RANGE = 1 << 3;

        const DEFAULT = Self::COLOR.bits()
        | Self::ALTITUDE.bits()
        | Self::RANGE.bits();
    }
}

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct FogStateC {
    name: Ascii<32>, // 00
    flags: u32,      // 32
    fog_type: u32,   // 36
    color: Color,    // 40
    altitude: Range, // 52
    range: Range,    // 60
}
impl_as_bytes!(FogStateC, 68);

impl ScriptObject for FogState {
    const INDEX: u8 = 28;
    const SIZE: u32 = FogStateC::SIZE;

    fn read(read: &mut CountingReader<impl Read>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("fog state size", size == Self::SIZE, read.offset)?;
        let fog_state: FogStateC = read.read_struct()?;

        let mut name = Ascii::zero();
        str_to_c_padded(DEFAULT_FOG_NAME, &mut name);
        assert_that!("fog state name", fog_state.name == name, read.prev + 0)?;

        assert_that!(
            "fog state flags",
            fog_state.flags == FogStateFlags::DEFAULT.bits(),
            read.prev + 32
        )?;
        assert_that!(
            "fog state type",
            fog_state.fog_type == FogType::Linear as u32,
            read.prev + 36
        )?;

        assert_color("fog state", &fog_state.color, read.prev + 40)?;

        Ok(Self {
            name: DEFAULT_FOG_NAME.to_owned(),
            fog_type: FogType::Linear,
            color: fog_state.color,
            altitude: fog_state.altitude,
            range: fog_state.range,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, _anim_def: &AnimDef) -> Result<()> {
        let mut name = Ascii::zero();
        str_to_c_padded(DEFAULT_FOG_NAME, &mut name);
        let fog_type = match &self.fog_type {
            FogType::Off => FogType::Off as u32,
            FogType::Linear => FogType::Linear as u32,
            FogType::Exponential => FogType::Exponential as u32,
        };
        write.write_struct(&FogStateC {
            name,
            flags: FogStateFlags::DEFAULT.bits(),
            fog_type,
            color: self.color,
            altitude: self.altitude,
            range: self.range,
        })?;
        Ok(())
    }
}
