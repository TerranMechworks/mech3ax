use super::utils::assert_color;
use super::ScriptObject;
use crate::AnimDef;
use mech3ax_api_types::{static_assert_size, Range, ReprSize as _, Vec3};
use mech3ax_common::io_ext::{CountingReader, WriteHelper};
use mech3ax_common::string::str_to_c_padded;
use mech3ax_common::{assert_that, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

const DEFAULT_FOG_NAME: &str = "default_fog_name";

bitflags::bitflags! {
    pub struct FogStateFlags: u32 {
        // const STATE = 1 << 0;
        const COLOR = 1 << 1;
        const ALTITUDE = 1 << 2;
        const RANGE = 1 << 3;

        const DEFAULT = Self::COLOR.bits
        | Self::ALTITUDE.bits
        | Self::RANGE.bits;
    }
}

#[repr(C)]
struct FogStateC {
    name: [u8; 32],  // 00
    flags: u32,      // 32
    fog_type: u32,   // 36
    color: Vec3,     // 40
    altitude: Range, // 52
    range: Range,    // 60
}
static_assert_size!(FogStateC, 68);

#[derive(Debug, Serialize, Deserialize)]
#[repr(u32)]
pub enum FogType {
    Off = 0,
    Linear = 1,
    Exponential = 2,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FogState {
    pub name: String,
    pub fog_type: FogType,
    pub color: Vec3,
    pub altitude: Range,
    pub range: Range,
}

impl ScriptObject for FogState {
    const INDEX: u8 = 28;
    const SIZE: u32 = FogStateC::SIZE;

    fn read<R: Read>(read: &mut CountingReader<R>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("fog state size", size == Self::SIZE, read.offset)?;
        let fog_state: FogStateC = read.read_struct()?;

        let mut name = [0; 32];
        str_to_c_padded(DEFAULT_FOG_NAME, &mut name);
        assert_that!("fog state name", fog_state.name == name, read.prev + 0)?;

        assert_that!(
            "fog state flags",
            fog_state.flags == FogStateFlags::DEFAULT.bits,
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

    fn write<W: Write>(&self, write: &mut W, _anim_def: &AnimDef) -> Result<()> {
        let mut name = [0; 32];
        str_to_c_padded(DEFAULT_FOG_NAME, &mut name);
        let fog_type = match &self.fog_type {
            FogType::Off => FogType::Off as u32,
            FogType::Linear => FogType::Linear as u32,
            FogType::Exponential => FogType::Exponential as u32,
        };
        write.write_struct(&FogStateC {
            name,
            flags: FogStateFlags::DEFAULT.bits,
            fog_type,
            color: self.color,
            altitude: self.altitude,
            range: self.range,
        })?;
        Ok(())
    }
}
