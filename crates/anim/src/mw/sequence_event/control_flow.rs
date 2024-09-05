use super::ScriptObject;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::{Callback, Else, ElseIf, EndIf, If, Loop};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, bool_c, Result};
use mech3ax_types::{impl_as_bytes, primitive_enum, AsBytes as _};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct LoopC {
    start: i32,
    loop_count: i32,
}
impl_as_bytes!(LoopC, 8);

impl ScriptObject for Loop {
    const INDEX: u8 = 30;
    const SIZE: u32 = LoopC::SIZE;

    fn read(read: &mut CountingReader<impl Read>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("loop size", size == LoopC::SIZE, read.offset)?;
        let loop_: LoopC = read.read_struct()?;
        assert_that!("loop start", loop_.start == 1, read.prev + 0)?;
        Ok(Loop {
            start: loop_.start,
            loop_count: loop_.loop_count,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, _anim_def: &AnimDef) -> Result<()> {
        write.write_struct(&LoopC {
            start: self.start,
            loop_count: self.loop_count,
        })?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct IfC {
    condition: u32,
    zero4: u32,
    value: [u8; 4],
}
impl_as_bytes!(IfC, 12);

primitive_enum! {
    enum Condition: u32 {
        RandomWeight = 1,
        PlayerRange = 2,
        AnimationLod = 4,
        HwRender = 32,
        PlayerFirstPerson = 64,
    }
}

impl ScriptObject for If {
    const INDEX: u8 = 31;
    const SIZE: u32 = IfC::SIZE;

    fn read(read: &mut CountingReader<impl Read>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("if size", size == Self::SIZE, read.offset)?;
        let if_: IfC = read.read_struct()?;

        let condition = assert_that!("if cond", enum Condition => if_.condition, read.prev + 0)?;
        assert_that!("if field 4", if_.zero4 == 0, read.prev + 4)?;

        match condition {
            Condition::RandomWeight => Ok(If::RandomWeight(f32::from_le_bytes(if_.value).into())),
            Condition::PlayerRange => Ok(If::PlayerRange(f32::from_le_bytes(if_.value).into())),
            Condition::AnimationLod => Ok(If::AnimationLod(u32::from_le_bytes(if_.value).into())),
            Condition::HwRender => {
                let value = u32::from_le_bytes(if_.value);
                let value = assert_that!("if value", bool value, read.prev + 8)?;
                Ok(If::HwRender(value.into()))
            }
            Condition::PlayerFirstPerson => {
                let value = u32::from_le_bytes(if_.value);
                let value = assert_that!("if value", bool value, read.prev + 8)?;
                Ok(If::PlayerFirstPerson(value.into()))
            }
        }
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, _anim_def: &AnimDef) -> Result<()> {
        let (condition, value) = match self {
            If::RandomWeight(value) => (Condition::RandomWeight as u32, value.to_le_bytes()),
            If::PlayerRange(value) => (Condition::PlayerRange as u32, value.to_le_bytes()),
            If::AnimationLod(value) => (Condition::AnimationLod as u32, value.to_le_bytes()),
            If::HwRender(value) => {
                let value: u32 = bool_c!(**value);
                (Condition::HwRender as u32, value.to_le_bytes())
            }
            If::PlayerFirstPerson(value) => {
                let value: u32 = bool_c!(**value);
                (Condition::PlayerFirstPerson as u32, value.to_le_bytes())
            }
        };
        write.write_struct(&IfC {
            condition,
            zero4: 0,
            value,
        })?;
        Ok(())
    }
}

impl ScriptObject for ElseIf {
    const INDEX: u8 = 33;
    const SIZE: u32 = IfC::SIZE;

    fn read(read: &mut CountingReader<impl Read>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("else if size", size == Self::SIZE, read.offset)?;
        let if_: IfC = read.read_struct()?;

        let condition =
            assert_that!("else if cond", enum Condition => if_.condition, read.prev + 0)?;
        assert_that!("else if field 4", if_.zero4 == 0, read.prev + 4)?;

        match condition {
            Condition::RandomWeight => {
                Ok(ElseIf::RandomWeight(f32::from_le_bytes(if_.value).into()))
            }
            Condition::PlayerRange => Ok(ElseIf::PlayerRange(f32::from_le_bytes(if_.value).into())),
            Condition::AnimationLod => {
                Ok(ElseIf::AnimationLod(u32::from_le_bytes(if_.value).into()))
            }
            Condition::HwRender => {
                let value = u32::from_le_bytes(if_.value);
                let value = assert_that!("else if value", bool value, read.prev + 8)?;
                Ok(ElseIf::HwRender(value.into()))
            }
            Condition::PlayerFirstPerson => {
                let value = u32::from_le_bytes(if_.value);
                let value = assert_that!("else if value", bool value, read.prev + 8)?;
                Ok(ElseIf::PlayerFirstPerson(value.into()))
            }
        }
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, _anim_def: &AnimDef) -> Result<()> {
        let (condition, value) = match self {
            ElseIf::RandomWeight(value) => (Condition::RandomWeight as u32, value.to_le_bytes()),
            ElseIf::PlayerRange(value) => (Condition::PlayerRange as u32, value.to_le_bytes()),
            ElseIf::AnimationLod(value) => (Condition::AnimationLod as u32, value.to_le_bytes()),
            ElseIf::HwRender(value) => {
                let value: u32 = bool_c!(**value);
                (Condition::HwRender as u32, value.to_le_bytes())
            }
            ElseIf::PlayerFirstPerson(value) => {
                let value: u32 = bool_c!(**value);
                (Condition::PlayerFirstPerson as u32, value.to_le_bytes())
            }
        };
        write.write_struct(&IfC {
            condition,
            zero4: 0,
            value,
        })?;
        Ok(())
    }
}

impl ScriptObject for Else {
    const INDEX: u8 = 32;
    const SIZE: u32 = 0;

    fn read(read: &mut CountingReader<impl Read>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("else size", size == Self::SIZE, read.offset)?;
        Ok(Self {})
    }

    fn write(&self, _write: &mut CountingWriter<impl Write>, _anim_def: &AnimDef) -> Result<()> {
        Ok(())
    }
}

impl ScriptObject for EndIf {
    const INDEX: u8 = 34;
    const SIZE: u32 = 0;

    fn read(read: &mut CountingReader<impl Read>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("end if size", size == Self::SIZE, read.offset)?;
        Ok(Self {})
    }

    fn write(&self, _write: &mut CountingWriter<impl Write>, _anim_def: &AnimDef) -> Result<()> {
        Ok(())
    }
}

impl ScriptObject for Callback {
    const INDEX: u8 = 35;
    const SIZE: u32 = 4;

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("callback size", size == Self::SIZE, read.offset)?;
        assert_that!(
            "anim def has callbacks",
            anim_def.has_callbacks == true,
            read.offset
        )?;
        let value = read.read_u32()?;
        Ok(Self { value })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, _anim_def: &AnimDef) -> Result<()> {
        write.write_u32(self.value)?;
        Ok(())
    }
}
