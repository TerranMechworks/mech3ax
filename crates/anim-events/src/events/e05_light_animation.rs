use super::EventAll;
use crate::types::{AnimDefLookup as _, Idx32};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::LightAnimation;
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::{Color, Range};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{impl_as_bytes, AsBytes as _, Ascii};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct LightAnimationC {
    light_name: Ascii<32>, // 00
    light_index: Idx32,    // 32
    range: Range,          // 36
    range_alt: Range,      // 44
    zero52: Range,         // 52
    color: Color,          // 60
    zero72: Color,         // 72
    zero84: Color,         // 84
    run_time: f32,         // 96
}
impl_as_bytes!(LightAnimationC, 100);

impl EventAll for LightAnimation {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(LightAnimationC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!(
            "light animation size",
            size == LightAnimationC::SIZE,
            read.offset
        )?;
        let light_anim: LightAnimationC = read.read_struct()?;

        assert_that!(
            "light anim run time",
            light_anim.run_time > 0.0,
            read.prev + 96
        )?;

        // not sure why this information is duplicated?
        let name = assert_utf8("light anim name", read.prev + 0, || {
            light_anim.light_name.to_str_padded()
        })?;
        let expected_name = anim_def.light_from_index(light_anim.light_index, read.prev + 32)?;
        assert_that!("light anim name", name == expected_name, read.prev + 32)?;

        if light_anim.range.min.to_bits() & 0x7FFF_FFFF == 0 {
            // min is +/- zero, so max could be anything
        } else if light_anim.range.min >= 0.0 {
            assert_that!(
                "light anim range far",
                light_anim.range.max >= light_anim.range.min,
                read.prev + 40
            )?;
        } else {
            assert_that!(
                "light anim range far",
                light_anim.range.max <= light_anim.range.min,
                read.prev + 40
            )?;
        }

        // there is a single "LIGHT_ANIMATION" named "pexp_light" in RC that
        // has this value set.
        let range_alt = if light_anim.range_alt == Range::DEFAULT {
            None
        } else {
            Some(light_anim.range_alt)
        };

        assert_that!(
            "light anim field 52",
            light_anim.zero52 == Range::DEFAULT,
            read.prev + 52
        )?;

        assert_that!("light anim color red", -5.0 <= light_anim.color.r <= 5.0, read.prev + 60)?;
        assert_that!("light anim color green", -5.0 <= light_anim.color.g <= 5.0, read.prev + 64)?;
        assert_that!("light anim color blue", -5.0 <= light_anim.color.b <= 5.0, read.prev + 68)?;

        assert_that!(
            "light anim field 72",
            light_anim.zero72 == Color::BLACK,
            read.prev + 72
        )?;
        assert_that!(
            "light anim field 84",
            light_anim.zero84 == Color::BLACK,
            read.prev + 84
        )?;

        Ok(Self {
            name,
            range: light_anim.range,
            color: light_anim.color,
            run_time: light_anim.run_time,
            range_alt,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let light_name = Ascii::from_str_padded(&self.name);
        let light_index = anim_def.light_to_index(&self.name)?;

        let light_anim = LightAnimationC {
            light_name,
            light_index,
            range: self.range,
            range_alt: self.range_alt.unwrap_or(Range::DEFAULT),
            zero52: Range::DEFAULT,
            color: self.color,
            zero72: Color::BLACK,
            zero84: Color::BLACK,
            run_time: self.run_time,
        };
        write.write_struct(&light_anim)?;
        Ok(())
    }
}
