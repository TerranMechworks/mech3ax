use super::EventAll;
use crate::utils::assert_color;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::{FogState, FogType};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::{Color, Range};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{bitflags, impl_as_bytes, AsBytes as _, Ascii, Maybe};
use std::io::{Read, Write};

const DEFAULT_FOG_NAME: Ascii<32> = Ascii::new(b"default_fog_name\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0");

bitflags! {
    struct FogStateFlags: u32 {
        const FOG_TYPE = 1 << 0;
        const COLOR = 1 << 1;
        const ALTITUDE = 1 << 2;
        const DENSITY = 1 << 3;
    }
}

type Flags = Maybe<u32, FogStateFlags>;
type FType = Maybe<u32, FogType>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct FogStateC {
    fog_name: Ascii<32>, // 00
    flags: Flags,        // 32
    ty_: FType,          // 36
    color: Color,        // 40
    altitude: Range,     // 52
    range: Range,        // 60
}
impl_as_bytes!(FogStateC, 68);

fn rc_m6_fixup(anim_def: &AnimDef, offset: usize) -> bool {
    anim_def.name == "vtol1" && anim_def.anim_name == "m6_start_animation" && offset == 460748
}

impl EventAll for FogState {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(FogStateC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("fog state size", size == FogStateC::SIZE, read.offset)?;
        let fog_state: FogStateC = read.read_struct()?;

        assert_that!(
            "fog state name",
            fog_state.fog_name == DEFAULT_FOG_NAME,
            read.prev + 0
        )?;

        let flags = assert_that!("fog state flags", flags fog_state.flags, read.prev + 32)?;

        let fog_type = if flags.contains(FogStateFlags::FOG_TYPE) {
            let value = assert_that!("fog state type", enum fog_state.ty_, read.prev + 36)?;
            Some(value)
        } else {
            let default_fog_type = if rc_m6_fixup(anim_def, read.prev) {
                log::debug!("fog state type fixup: linear -> off");
                FogType::Off.maybe()
            } else {
                FogType::Linear.maybe()
            };

            assert_that!(
                "fog state type",
                fog_state.ty_ == default_fog_type,
                read.prev + 36
            )?;
            None
        };

        let color = if flags.contains(FogStateFlags::COLOR) {
            assert_color!("fog state color", fog_state.color, read.prev + 40)?;
            Some(fog_state.color)
        } else {
            assert_that!(
                "fog state color",
                fog_state.color == Color::BLACK,
                read.prev + 40
            )?;
            None
        };

        let altitude = if flags.contains(FogStateFlags::ALTITUDE) {
            // the altitude range just misbehaves for:
            // MW C2 and C4B: `horizon`, `hormain`
            Some(fog_state.altitude)
        } else {
            assert_that!(
                "fog state altitude",
                fog_state.altitude == Range::DEFAULT,
                read.prev + 52
            )?;
            None
        };

        let range = if flags.contains(FogStateFlags::DENSITY) {
            assert_that!(
                "fog state range near",
                fog_state.range.min > 0.0,
                read.prev + 60
            )?;
            assert_that!(
                "fog state range near",
                fog_state.range.max >= fog_state.range.min,
                read.prev + 64
            )?;
            Some(fog_state.range)
        } else {
            assert_that!(
                "fog state range",
                fog_state.range == Range::DEFAULT,
                read.prev + 60
            )?;
            None
        };

        Ok(Self {
            type_: fog_type,
            color,
            altitude,
            range,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let mut flags = FogStateFlags::empty();

        if self.type_.is_some() {
            flags |= FogStateFlags::FOG_TYPE;
        }
        if self.color.is_some() {
            flags |= FogStateFlags::COLOR;
        }
        if self.altitude.is_some() {
            flags |= FogStateFlags::ALTITUDE;
        }
        if self.range.is_some() {
            flags |= FogStateFlags::DENSITY;
        }

        let default_fog_type = if rc_m6_fixup(anim_def, write.offset) {
            log::debug!("fog state type fixup: linear -> off");
            FogType::Off
        } else {
            FogType::Linear
        };

        let fog_state = FogStateC {
            fog_name: DEFAULT_FOG_NAME,
            flags: flags.maybe(),
            ty_: self.type_.unwrap_or(default_fog_type).maybe(),
            color: self.color.unwrap_or(Color::BLACK),
            altitude: self.altitude.unwrap_or(Range::DEFAULT),
            range: self.range.unwrap_or(Range::DEFAULT),
        };
        write.write_struct(&fog_state)?;
        Ok(())
    }
}
