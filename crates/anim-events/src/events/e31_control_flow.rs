use super::ScriptObject;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::{Else, ElseIf, EndIf, If};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, assert_with_msg, Result};
use mech3ax_types::{impl_as_bytes, primitive_enum, AsBytes as _, Maybe};
use std::io::{Read, Write};

macro_rules! bool_c {
    ($value:expr) => {
        if $value {
            1u32
        } else {
            0u32
        }
    };
}

macro_rules! assert_bool {
    ($name:literal, bool $v:ident, $pos:expr) => {
        match $v {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(assert_with_msg!(
                "Expected `{}` to be 0 or 1, but was {} (at {})",
                $name,
                $v,
                $pos
            )),
        }
    };
}

primitive_enum! {
    enum Condition: u32 {
        RandomWeight = 1,
        PlayerRange = 2,
        AnimationLod = 4,
        HwRender = 32,
        PlayerFirstPerson = 64,
    }
}

type Cond = Maybe<u32, Condition>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct IfC {
    condition: Cond,
    zero4: u32,
    value: [u8; 4],
}
impl_as_bytes!(IfC, 12);

impl ScriptObject for If {
    const INDEX: u8 = 31;
    const SIZE: u32 = IfC::SIZE;

    fn read(read: &mut CountingReader<impl Read>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("if size", size == Self::SIZE, read.offset)?;
        let if_: IfC = read.read_struct()?;

        let condition = assert_that!("if cond", enum if_.condition, read.prev + 0)?;
        assert_that!("if field 4", if_.zero4 == 0, read.prev + 4)?;

        match condition {
            Condition::RandomWeight => Ok(If::RandomWeight(f32::from_le_bytes(if_.value).into())),
            Condition::PlayerRange => Ok(If::PlayerRange(f32::from_le_bytes(if_.value).into())),
            Condition::AnimationLod => Ok(If::AnimationLod(u32::from_le_bytes(if_.value).into())),
            Condition::HwRender => {
                let value = u32::from_le_bytes(if_.value);
                let value = assert_bool!("if value", bool value, read.prev + 8)?;
                Ok(If::HwRender(value.into()))
            }
            Condition::PlayerFirstPerson => {
                let value = u32::from_le_bytes(if_.value);
                let value = assert_bool!("if value", bool value, read.prev + 8)?;
                Ok(If::PlayerFirstPerson(value.into()))
            }
        }
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, _anim_def: &AnimDef) -> Result<()> {
        let (condition, value) = match self {
            If::RandomWeight(value) => (Condition::RandomWeight, value.to_le_bytes()),
            If::PlayerRange(value) => (Condition::PlayerRange, value.to_le_bytes()),
            If::AnimationLod(value) => (Condition::AnimationLod, value.to_le_bytes()),
            If::HwRender(value) => {
                let value: u32 = bool_c!(**value);
                (Condition::HwRender, value.to_le_bytes())
            }
            If::PlayerFirstPerson(value) => {
                let value: u32 = bool_c!(**value);
                (Condition::PlayerFirstPerson, value.to_le_bytes())
            }
        };
        write.write_struct(&IfC {
            condition: condition.maybe(),
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

        let condition = assert_that!("else if cond", enum if_.condition, read.prev + 0)?;
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
                let value = assert_bool!("else if value", bool value, read.prev + 8)?;
                Ok(ElseIf::HwRender(value.into()))
            }
            Condition::PlayerFirstPerson => {
                let value = u32::from_le_bytes(if_.value);
                let value = assert_bool!("else if value", bool value, read.prev + 8)?;
                Ok(ElseIf::PlayerFirstPerson(value.into()))
            }
        }
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, _anim_def: &AnimDef) -> Result<()> {
        let (condition, value) = match self {
            ElseIf::RandomWeight(value) => (Condition::RandomWeight, value.to_le_bytes()),
            ElseIf::PlayerRange(value) => (Condition::PlayerRange, value.to_le_bytes()),
            ElseIf::AnimationLod(value) => (Condition::AnimationLod, value.to_le_bytes()),
            ElseIf::HwRender(value) => {
                let value: u32 = bool_c!(**value);
                (Condition::HwRender, value.to_le_bytes())
            }
            ElseIf::PlayerFirstPerson(value) => {
                let value: u32 = bool_c!(**value);
                (Condition::PlayerFirstPerson, value.to_le_bytes())
            }
        };
        write.write_struct(&IfC {
            condition: condition.maybe(),
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
