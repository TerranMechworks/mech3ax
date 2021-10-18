use super::ScriptObject;
use crate::AnimDef;
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, WriteHelper};
use mech3ax_common::size::ReprSize;
use mech3ax_common::string::{str_from_c_padded, str_to_c_padded};
use mech3ax_common::{assert_that, static_assert_size, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[repr(C)]
struct AnimationC {
    name: [u8; 32],
    zero32: u32,
}
static_assert_size!(AnimationC, 36);

fn read_animation<R: Read>(read: &mut CountingReader<R>) -> Result<String> {
    let animation: AnimationC = read.read_struct()?;
    let name = assert_utf8("animation name", read.prev + 0, || {
        str_from_c_padded(&animation.name)
    })?;
    assert_that!("animation field 32", animation.zero32 == 0, read.prev + 32)?;
    Ok(name)
}

fn write_animation<W: Write>(write: &mut W, name: &str) -> Result<()> {
    let mut fill = [0; 32];
    str_to_c_padded(name, &mut fill);
    write.write_struct(&AnimationC {
        name: fill,
        zero32: 0,
    })?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StopAnimation {
    pub name: String,
}

impl ScriptObject for StopAnimation {
    const INDEX: u8 = 25;
    const SIZE: u32 = AnimationC::SIZE;

    fn read<R: Read>(read: &mut CountingReader<R>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("stop animation size", size == Self::SIZE, read.offset)?;
        let name = read_animation(read)?;
        Ok(Self { name })
    }

    fn write<W: Write>(&self, write: &mut W, _anim_def: &AnimDef) -> Result<()> {
        write_animation(write, &self.name)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResetAnimation {
    pub name: String,
}

impl ScriptObject for ResetAnimation {
    const INDEX: u8 = 26;
    const SIZE: u32 = AnimationC::SIZE;

    fn read<R: Read>(read: &mut CountingReader<R>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("reset animation size", size == Self::SIZE, read.offset)?;
        let name = read_animation(read)?;
        Ok(Self { name })
    }

    fn write<W: Write>(&self, write: &mut W, _anim_def: &AnimDef) -> Result<()> {
        write_animation(write, &self.name)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvalidateAnimation {
    pub name: String,
}

impl ScriptObject for InvalidateAnimation {
    const INDEX: u8 = 27;
    const SIZE: u32 = AnimationC::SIZE;

    fn read<R: Read>(read: &mut CountingReader<R>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("invalidate animation size", size == Self::SIZE, read.offset)?;
        let name = read_animation(read)?;
        Ok(Self { name })
    }

    fn write<W: Write>(&self, write: &mut W, _anim_def: &AnimDef) -> Result<()> {
        write_animation(write, &self.name)
    }
}
