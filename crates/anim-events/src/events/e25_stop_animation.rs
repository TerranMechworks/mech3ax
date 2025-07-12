use super::{EventMw, EventPm, EventRc};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::anim::events::StopAnimation;
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{Result, assert_that};
use mech3ax_types::{AsBytes as _, Ascii, impl_as_bytes};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct StopAnimationC {
    name: Ascii<32>,     // 00
    anim_ref_index: i16, // 32
    anim_def_index: i16, // 34
}
impl_as_bytes!(StopAnimationC, 36);

fn read_animation_pg(read: &mut CountingReader<impl Read>) -> Result<String> {
    let animation: StopAnimationC = read.read_struct()?;

    let name = assert_utf8("stop animation name", read.prev + 0, || {
        animation.name.to_str_padded()
    })?;
    assert_that!(
        "stop animation ref index",
        animation.anim_ref_index == 0,
        read.prev + 32
    )?;
    assert_that!(
        "stop animation def index",
        animation.anim_def_index == 0,
        read.prev + 34
    )?;
    Ok(name)
}

fn write_animation_pg(write: &mut CountingWriter<impl Write>, name: &str) -> Result<()> {
    let name = Ascii::from_str_padded(name);

    let animation = StopAnimationC {
        name,
        anim_ref_index: 0,
        anim_def_index: 0,
    };
    write.write_struct(&animation)?;
    Ok(())
}

fn read_animation_pm(read: &mut CountingReader<impl Read>) -> Result<String> {
    let animation: StopAnimationC = read.read_struct()?;
    let name = assert_utf8("stop animation name", read.prev + 0, || {
        animation.name.to_str_padded()
    })?;
    assert_that!(
        "stop animation ref index",
        animation.anim_ref_index == -1,
        read.prev + 32
    )?;
    assert_that!(
        "stop animation def index",
        animation.anim_def_index == -1,
        read.prev + 34
    )?;
    Ok(name)
}

fn write_animation_pm(write: &mut CountingWriter<impl Write>, name: &str) -> Result<()> {
    let name = Ascii::from_str_padded(name);
    let animation = StopAnimationC {
        name,
        anim_ref_index: -1,
        anim_def_index: -1,
    };
    write.write_struct(&animation)?;
    Ok(())
}

impl EventMw for StopAnimation {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(StopAnimationC::SIZE)
    }

    #[inline]
    fn read(read: &mut CountingReader<impl Read>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!(
            "stop animation size",
            size == StopAnimationC::SIZE,
            read.offset
        )?;
        let name = read_animation_pg(read)?;
        Ok(Self { name })
    }

    #[inline]
    fn write(&self, write: &mut CountingWriter<impl Write>, _anim_def: &AnimDef) -> Result<()> {
        write_animation_pg(write, &self.name)
    }
}

impl EventRc for StopAnimation {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(StopAnimationC::SIZE)
    }

    #[inline]
    fn read(read: &mut CountingReader<impl Read>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!(
            "stop animation size",
            size == StopAnimationC::SIZE,
            read.offset
        )?;
        let name = read_animation_pg(read)?;
        Ok(Self { name })
    }

    #[inline]
    fn write(&self, write: &mut CountingWriter<impl Write>, _anim_def: &AnimDef) -> Result<()> {
        write_animation_pg(write, &self.name)
    }
}

impl EventPm for StopAnimation {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(StopAnimationC::SIZE)
    }

    #[inline]
    fn read(read: &mut CountingReader<impl Read>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!(
            "stop animation size",
            size == StopAnimationC::SIZE,
            read.offset
        )?;
        let name = read_animation_pm(read)?;
        Ok(Self { name })
    }

    #[inline]
    fn write(&self, write: &mut CountingWriter<impl Write>, _anim_def: &AnimDef) -> Result<()> {
        write_animation_pm(write, &self.name)
    }
}
