use super::EventAll;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::anim::events::{InvalidateAnimation, ResetAnimation};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{Result, assert_that};
use mech3ax_types::{AsBytes as _, Ascii, impl_as_bytes};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct AnimationC {
    name: Ascii<32>,
    anim_def_index: i32,
}
impl_as_bytes!(AnimationC, 36);

fn read_animation(read: &mut CountingReader<impl Read>) -> Result<String> {
    let animation: AnimationC = read.read_struct()?;

    let name = assert_utf8("animation name", read.prev + 0, || {
        animation.name.to_str_padded()
    })?;
    assert_that!(
        "animation index",
        animation.anim_def_index == 0,
        read.prev + 32
    )?;
    Ok(name)
}

fn write_animation(write: &mut CountingWriter<impl Write>, name: &str) -> Result<()> {
    let name = Ascii::from_str_padded(name);

    let animation = AnimationC {
        name,
        anim_def_index: 0,
    };
    write.write_struct(&animation)?;
    Ok(())
}

impl EventAll for ResetAnimation {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(AnimationC::SIZE)
    }

    #[inline]
    fn read(read: &mut CountingReader<impl Read>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!(
            "reset animation size",
            size == AnimationC::SIZE,
            read.offset
        )?;
        let name = read_animation(read)?;
        Ok(Self { name })
    }

    #[inline]
    fn write(&self, write: &mut CountingWriter<impl Write>, _anim_def: &AnimDef) -> Result<()> {
        write_animation(write, &self.name)
    }
}

impl EventAll for InvalidateAnimation {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(AnimationC::SIZE)
    }

    #[inline]
    fn read(read: &mut CountingReader<impl Read>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!(
            "invalidate animation size",
            size == AnimationC::SIZE,
            read.offset
        )?;
        let name = read_animation(read)?;
        Ok(Self { name })
    }

    #[inline]
    fn write(&self, write: &mut CountingWriter<impl Write>, _anim_def: &AnimDef) -> Result<()> {
        write_animation(write, &self.name)
    }
}
