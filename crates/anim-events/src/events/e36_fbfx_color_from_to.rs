use super::delta::{dec_f32, delta};
use super::ScriptObject;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::{FrameBufferEffectColor, Rgba};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{impl_as_bytes, AsBytes as _};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct FbFxColorFromToC {
    from_red: f32,
    to_red: f32,
    delta_red: f32,
    from_green: f32,
    to_green: f32,
    delta_green: f32,
    from_blue: f32,
    to_blue: f32,
    delta_blue: f32,
    from_alpha: f32,
    to_alpha: f32,
    delta_alpha: f32,
    runtime: f32,
}
impl_as_bytes!(FbFxColorFromToC, 52);

impl ScriptObject for FrameBufferEffectColor {
    const INDEX: u8 = 36;
    const SIZE: u32 = FbFxColorFromToC::SIZE;

    fn read(read: &mut CountingReader<impl Read>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("fbfx color from to size", size == Self::SIZE, read.offset)?;
        let fbfx: FbFxColorFromToC = read.read_struct()?;

        assert_that!("fbfx color from red", 0.0 <= fbfx.from_red <= 1.0, read.prev + 0)?;
        assert_that!("fbfx color to red", 0.0 <= fbfx.to_red <= 1.0, read.prev + 4)?;
        assert_that!("fbfx color from green", 0.0 <= fbfx.from_green <= 1.0, read.prev + 12)?;
        assert_that!("fbfx color to green", 0.0 <= fbfx.to_green <= 1.0, read.prev + 16)?;
        assert_that!("fbfx color from blue", 0.0 <= fbfx.from_blue <= 1.0, read.prev + 24)?;
        assert_that!("fbfx color to blue", 0.0 <= fbfx.to_blue <= 1.0, read.prev + 28)?;
        assert_that!("fbfx color from alpha", 0.0 <= fbfx.from_alpha <= 1.0, read.prev + 36)?;
        assert_that!("fbfx color to alpha", 0.0 <= fbfx.to_alpha <= 1.0, read.prev + 40)?;
        assert_that!("fbfx color runtime", fbfx.runtime > 0.0, read.prev + 48)?;

        let delta_red = delta(fbfx.to_red, fbfx.from_red, fbfx.runtime);
        assert_that!(
            "fbfx color delta red",
            fbfx.delta_red == delta_red,
            read.prev + 8
        )?;
        let delta_green = delta(fbfx.to_green, fbfx.from_green, fbfx.runtime);
        assert_that!(
            "fbfx color delta green",
            fbfx.delta_green == delta_green,
            read.prev + 20
        )?;
        let delta_blue = delta(fbfx.to_blue, fbfx.from_blue, fbfx.runtime);
        assert_that!(
            "fbfx color delta blue",
            fbfx.delta_blue == delta_blue,
            read.prev + 32
        )?;
        let mut delta_alpha = delta(fbfx.to_alpha, fbfx.from_alpha, fbfx.runtime);

        #[allow(clippy::float_cmp)]
        let fudge_alpha = if fbfx.delta_alpha != delta_alpha {
            // two values in c3/anim.zbd of v1.0-us-pre and v1.1-us-pre have slightly
            // incorrect values here - off by the last bit (out of ~1800)
            //
            // to       0x3EA3D70A 0b0 01111101 01000111101011100001010 0.32
            // from     0x00000000 0b0 00000000 00000000000000000000000 0
            // runtime  0x3F400000 0b0 01111110 10000000000000000000000 0.75
            // expected 0x3EDA740D 0b0 01111101 10110100111010000001101 0.42666665
            // actual   0x3EDA740E 0b0 01111101 10110100111010000001110 0.42666668
            // adjusted 0x3F3FFFFF 0b0 01111110 01111111111111111111111 0.74999994
            //
            // to       0x00000000 0b0 00000000 00000000000000000000000 0
            // from     0x3EA3D70A 0b0 01111101 01000111101011100001010 0.32
            // runtime  0x3FC00000 0b0 01111111 10000000000000000000000 1.5
            // expected 0xBE5A740D 0b1 01111100 10110100111010000001101 -0.21333332
            // actual   0xBE5A740E 0b1 01111100 10110100111010000001110 -0.21333334
            // adjusted 0x3FBFFFFF 0b0 01111111 01111111111111111111111 1.4999999
            //
            // note that other events with the same values have the correctly calculated deltas!
            delta_alpha = delta(fbfx.to_alpha, fbfx.from_alpha, dec_f32(fbfx.runtime));
            true
        } else {
            false
        };
        assert_that!(
            "fbfx color delta alpha",
            fbfx.delta_alpha == delta_alpha,
            read.prev + 44
        )?;

        Ok(Self {
            from: Rgba {
                r: fbfx.from_red,
                g: fbfx.from_green,
                b: fbfx.from_blue,
                a: fbfx.from_alpha,
            },
            to: Rgba {
                r: fbfx.to_red,
                g: fbfx.to_green,
                b: fbfx.to_blue,
                a: fbfx.to_alpha,
            },
            runtime: fbfx.runtime,
            fudge_alpha,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, _anim_def: &AnimDef) -> Result<()> {
        let delta_red = delta(self.to.r, self.from.r, self.runtime);
        let delta_green = delta(self.to.g, self.from.g, self.runtime);
        let delta_blue = delta(self.to.b, self.from.b, self.runtime);
        let runtime = if self.fudge_alpha {
            dec_f32(self.runtime)
        } else {
            self.runtime
        };
        let delta_alpha = delta(self.to.a, self.from.a, runtime);
        write.write_struct(&FbFxColorFromToC {
            from_red: self.from.r,
            to_red: self.to.r,
            delta_red,
            from_green: self.from.g,
            to_green: self.to.g,
            delta_green,
            from_blue: self.from.b,
            to_blue: self.to.b,
            delta_blue,
            from_alpha: self.from.a,
            to_alpha: self.to.a,
            delta_alpha,
            runtime: self.runtime,
        })?;
        Ok(())
    }
}
