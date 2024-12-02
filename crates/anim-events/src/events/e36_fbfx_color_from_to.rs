use super::delta::{delta, FloatFromToC};
use super::EventAll;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::{FbfxColorFromTo, Rgba};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{impl_as_bytes, AsBytes as _};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct FbFxColorFromToC {
    red: FloatFromToC,   // 00
    green: FloatFromToC, // 12
    blue: FloatFromToC,  // 24
    alpha: FloatFromToC, // 36
    run_time: f32,       // 48
}
impl_as_bytes!(FbFxColorFromToC, 52);

impl EventAll for FbfxColorFromTo {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(FbFxColorFromToC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!(
            "fbfx color size",
            size == FbFxColorFromToC::SIZE,
            read.offset
        )?;
        let fbfx: FbFxColorFromToC = read.read_struct()?;

        assert_that!("fbfx color run time", fbfx.run_time > 0.0, read.prev + 48)?;
        let run_time = fbfx.run_time;

        assert_that!("fbfx color from red", 0.0 <= fbfx.red.from <= 1.0, read.prev + 0)?;
        assert_that!("fbfx color to red", 0.0 <= fbfx.red.to <= 1.0, read.prev + 4)?;
        // TODO: rc
        // assert_that!("fbfx color from green", 0.0 <= fbfx.green.from <= 1.0, read.prev + 12)?;
        assert_that!("fbfx color to green", 0.0 <= fbfx.green.to <= 1.0, read.prev + 16)?;
        // TODO: rc
        // assert_that!("fbfx color from blue", 0.0 <= fbfx.blue.from <= 1.0, read.prev + 24)?;
        assert_that!("fbfx color to blue", 0.0 <= fbfx.blue.to <= 1.0, read.prev + 28)?;
        assert_that!("fbfx color from alpha", 0.0 <= fbfx.alpha.from <= 1.0, read.prev + 36)?;
        assert_that!("fbfx color to alpha", 0.0 <= fbfx.alpha.to <= 1.0, read.prev + 40)?;

        let delta_red = delta(fbfx.red.from, fbfx.red.to, run_time);
        assert_that!(
            "fbfx color delta red",
            fbfx.red.delta == delta_red,
            read.prev + 8
        )?;
        let delta_green = delta(fbfx.green.from, fbfx.green.to, run_time);
        assert_that!(
            "fbfx color delta green",
            fbfx.green.delta == delta_green,
            read.prev + 20
        )?;
        let delta_blue = delta(fbfx.blue.from, fbfx.blue.to, run_time);
        assert_that!(
            "fbfx color delta blue",
            fbfx.blue.delta == delta_blue,
            read.prev + 32
        )?;

        // TODO
        let alpha_delta = delta(fbfx.alpha.from, fbfx.alpha.to, run_time);
        let alpha_delta = if fbfx.alpha.delta == alpha_delta {
            log::trace!("fbfx color alpha delta: OK");
            None
        } else {
            log::debug!("fbfx color alpha delta: FAIL");
            log::error!(
                "DELTA VAL FAIL: `{}`, `{}`",
                anim_def.anim_name,
                anim_def.anim_root_name
            );
            Some(fbfx.alpha.delta)
        };
        // #[allow(clippy::float_cmp)]
        // let fudge_alpha = if fbfx.alpha.delta != delta_alpha {
        //     // two values in c3/anim.zbd of v1.0-us-pre and v1.1-us-pre have slightly
        //     // incorrect values here - off by the last bit (out of ~1800)
        //     //
        //     // to       0x3EA3D70A 0b0 01111101 01000111101011100001010 0.32
        //     // from     0x00000000 0b0 00000000 00000000000000000000000 0
        //     // run time 0x3F400000 0b0 01111110 10000000000000000000000 0.75
        //     // expected 0x3EDA740D 0b0 01111101 10110100111010000001101 0.42666665
        //     // actual   0x3EDA740E 0b0 01111101 10110100111010000001110 0.42666668
        //     // adjusted 0x3F3FFFFF 0b0 01111110 01111111111111111111111 0.74999994
        //     //
        //     // to       0x00000000 0b0 00000000 00000000000000000000000 0
        //     // from     0x3EA3D70A 0b0 01111101 01000111101011100001010 0.32
        //     // run time 0x3FC00000 0b0 01111111 10000000000000000000000 1.5
        //     // expected 0xBE5A740D 0b1 01111100 10110100111010000001101 -0.21333332
        //     // actual   0xBE5A740E 0b1 01111100 10110100111010000001110 -0.21333334
        //     // adjusted 0x3FBFFFFF 0b0 01111111 01111111111111111111111 1.4999999
        //     //
        //     // note that other events with the same values have the correctly calculated deltas!
        //     delta_alpha = delta(fbfx.alpha.to, fbfx.alpha.from, dec_f32(run_time));
        //     true
        // } else {
        //     false
        // };
        // assert_that!(
        //     "fbfx color delta alpha",
        //     fbfx.alpha.delta == delta_alpha,
        //     read.prev + 44
        // )?;

        Ok(Self {
            from: Rgba {
                r: fbfx.red.from,
                g: fbfx.green.from,
                b: fbfx.blue.from,
                a: fbfx.alpha.from,
            },
            to: Rgba {
                r: fbfx.red.to,
                g: fbfx.green.to,
                b: fbfx.blue.to,
                a: fbfx.alpha.to,
            },
            run_time,
            alpha_delta,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, _anim_def: &AnimDef) -> Result<()> {
        let run_time = self.run_time;

        // TODO
        // let run_time = if self.fudge_alpha {
        //     dec_f32(self.run_time)
        // } else {
        //     self.run_time
        // };
        let delta_alpha = self
            .alpha_delta
            .unwrap_or_else(|| delta(self.from.a, self.to.a, run_time));
        let delta = Rgba {
            r: delta(self.from.r, self.to.r, run_time),
            g: delta(self.from.g, self.to.g, run_time),
            b: delta(self.from.b, self.to.b, run_time),
            a: delta_alpha,
        };

        let fbfx = FbFxColorFromToC {
            red: FloatFromToC {
                from: self.from.r,
                to: self.to.r,
                delta: delta.r,
            },
            green: FloatFromToC {
                from: self.from.g,
                to: self.to.g,
                delta: delta.g,
            },
            blue: FloatFromToC {
                from: self.from.b,
                to: self.to.b,
                delta: delta.b,
            },
            alpha: FloatFromToC {
                from: self.from.a,
                to: self.to.a,
                delta: delta.a,
            },
            run_time,
        };
        write.write_struct(&fbfx)?;
        Ok(())
    }
}
