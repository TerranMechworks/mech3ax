use super::ScriptObject;
use crate::anim::AnimDef;
use crate::io_ext::{CountingReader, WriteHelper};
use crate::size::ReprSize;
use crate::types::Vec4;
use crate::{assert_that, static_assert_size, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

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
static_assert_size!(FbFxColorFromToC, 52);

#[derive(Debug, Serialize, Deserialize)]
pub struct FrameBufferEffectColor {
    pub from: Vec4,
    pub to: Vec4,
    pub delta: Vec4,
    pub runtime: f32,
}

impl ScriptObject for FrameBufferEffectColor {
    const INDEX: u8 = 36;
    const SIZE: u32 = FbFxColorFromToC::SIZE;

    fn read<R: Read>(read: &mut CountingReader<R>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
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
        // TODO: can the delta's be calculated? should be (to - from) / runtime
        assert_that!("fbfx color runtime", fbfx.runtime > 0.0, read.prev + 48)?;

        Ok(Self {
            from: Vec4(
                fbfx.from_red,
                fbfx.from_green,
                fbfx.from_blue,
                fbfx.from_alpha,
            ),
            to: Vec4(fbfx.to_red, fbfx.to_green, fbfx.to_blue, fbfx.to_alpha),
            delta: Vec4(
                fbfx.delta_red,
                fbfx.delta_green,
                fbfx.delta_blue,
                fbfx.delta_alpha,
            ),
            runtime: fbfx.runtime,
        })
    }

    fn write<W: Write>(&self, write: &mut W, _anim_def: &AnimDef) -> Result<()> {
        write.write_struct(&FbFxColorFromToC {
            from_red: self.from.0,
            to_red: self.to.0,
            delta_red: self.delta.0,
            from_green: self.from.1,
            to_green: self.to.1,
            delta_green: self.delta.1,
            from_blue: self.from.2,
            to_blue: self.to.2,
            delta_blue: self.delta.2,
            from_alpha: self.from.3,
            to_alpha: self.to.3,
            delta_alpha: self.delta.3,
            runtime: self.runtime,
        })?;
        Ok(())
    }
}
