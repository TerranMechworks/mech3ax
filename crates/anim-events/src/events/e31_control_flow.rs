use super::{EventAll, EventMw, EventPm, EventRc};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::anim::events::{Condition, Else, Elseif, Endif, If, NodeUndercover};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{Result, assert_that, assert_with_msg};
use mech3ax_types::{AsBytes as _, Bytes, Maybe, impl_as_bytes, primitive_enum};
use std::io::{Read, Write};

macro_rules! bool_c {
    ($value:expr) => {
        if $value { 1u32 } else { 0u32 }
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
    enum ConditionType: u32 {
        RandomWeight = 1,           // 0x00001
        PlayerRange = 2,            // 0x00002
        AnimationLod = 4,           // 0x00004
        NodeUndercover = 16,        // 0x00010
        HwRender = 32,              // 0x00020
        PlayerFirstPerson = 64,     // 0x00040
    }
}

type CondType = Maybe<u32, ConditionType>;

fn assert_cond(
    cond_type: ConditionType,
    node_index: u32,
    value: [u8; 4],
    offset: usize,
) -> Result<Condition> {
    match cond_type {
        ConditionType::RandomWeight => {
            assert_that!("condition node index", node_index == 0, offset + 4)?;
            Ok(Condition::RandomWeight(f32::from_le_bytes(value)))
        }
        ConditionType::PlayerRange => {
            assert_that!("condition node index", node_index == 0, offset + 4)?;
            Ok(Condition::PlayerRange(f32::from_le_bytes(value)))
        }
        ConditionType::AnimationLod => {
            assert_that!("condition node index", node_index == 0, offset + 4)?;
            Ok(Condition::AnimationLod(u32::from_le_bytes(value)))
        }
        ConditionType::NodeUndercover => {
            let distance = u32::from_le_bytes(value);
            Ok(Condition::NodeUndercover(NodeUndercover {
                node_index,
                distance,
            }))
        }
        ConditionType::HwRender => {
            assert_that!("condition node index", node_index == 0, offset + 4)?;
            let value = u32::from_le_bytes(value);
            let value = assert_bool!("condition value", bool value, offset + 8)?;
            Ok(Condition::HwRender(value))
        }
        ConditionType::PlayerFirstPerson => {
            assert_that!("condition node index", node_index == 0, offset + 4)?;
            let value = u32::from_le_bytes(value);
            let value = assert_bool!("condition value", bool value, offset + 8)?;
            Ok(Condition::PlayerFirstPerson(value))
        }
    }
}

fn make_cond(condition: &Condition) -> (ConditionType, u32, [u8; 4]) {
    match condition {
        Condition::RandomWeight(value) => (ConditionType::RandomWeight, 0, value.to_le_bytes()),
        Condition::PlayerRange(value) => (ConditionType::PlayerRange, 0, value.to_le_bytes()),
        Condition::AnimationLod(value) => (ConditionType::AnimationLod, 0, value.to_le_bytes()),
        Condition::NodeUndercover(value) => (
            ConditionType::NodeUndercover,
            value.node_index,
            value.distance.to_le_bytes(),
        ),
        Condition::HwRender(value) => (ConditionType::HwRender, 0, bool_c!(*value).to_le_bytes()),
        Condition::PlayerFirstPerson(value) => (
            ConditionType::PlayerFirstPerson,
            0,
            bool_c!(*value).to_le_bytes(),
        ),
    }
}

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct IfPgC {
    cond_type: CondType,
    node_index: u32,
    value: Bytes<4>,
}
impl_as_bytes!(IfPgC, 12);

fn read_if_pg(read: &mut CountingReader<impl Read>, _anim_def: &AnimDef, size: u32) -> Result<If> {
    assert_that!("if size", size == IfPgC::SIZE, read.offset)?;
    let if_: IfPgC = read.read_struct()?;

    let cond_type = assert_that!("if cond", enum if_.cond_type, read.prev + 0)?;
    let value: [u8; 4] = if_.value.into();

    let condition = assert_cond(cond_type, if_.node_index, value, read.prev)?;
    Ok(If { condition })
}

fn write_if_pg(
    if_: &If,
    write: &mut CountingWriter<impl Write>,
    _anim_def: &AnimDef,
) -> Result<()> {
    let (cond_type, zero4, value) = make_cond(&if_.condition);

    let if_ = IfPgC {
        cond_type: cond_type.maybe(),
        node_index: zero4,
        value: value.into(),
    };
    write.write_struct(&if_)?;
    Ok(())
}

impl EventMw for If {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(IfPgC::SIZE)
    }

    #[inline]
    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        read_if_pg(read, anim_def, size)
    }

    #[inline]
    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        write_if_pg(self, write, anim_def)
    }
}

impl EventRc for If {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(IfPgC::SIZE)
    }

    #[inline]
    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        read_if_pg(read, anim_def, size)
    }

    #[inline]
    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        write_if_pg(self, write, anim_def)
    }
}

fn read_elseif_pg(
    read: &mut CountingReader<impl Read>,
    _anim_def: &AnimDef,
    size: u32,
) -> Result<Elseif> {
    assert_that!("else if size", size == IfPgC::SIZE, read.offset)?;
    let if_: IfPgC = read.read_struct()?;

    let cond_type = assert_that!("else if cond", enum if_.cond_type, read.prev + 0)?;
    let value: [u8; 4] = if_.value.into_inner();

    let condition = assert_cond(cond_type, if_.node_index, value, read.prev)?;
    Ok(Elseif { condition })
}

fn write_elseif_pg(
    if_: &Elseif,
    write: &mut CountingWriter<impl Write>,
    _anim_def: &AnimDef,
) -> Result<()> {
    let (cond_type, zero4, value) = make_cond(&if_.condition);

    let if_ = IfPgC {
        cond_type: cond_type.maybe(),
        node_index: zero4,
        value: value.into(),
    };
    write.write_struct(&if_)?;
    Ok(())
}

impl EventMw for Elseif {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(IfPgC::SIZE)
    }

    #[inline]
    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        read_elseif_pg(read, anim_def, size)
    }

    #[inline]
    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        write_elseif_pg(self, write, anim_def)
    }
}

impl EventRc for Elseif {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(IfPgC::SIZE)
    }

    #[inline]
    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        read_elseif_pg(read, anim_def, size)
    }

    #[inline]
    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        write_elseif_pg(self, write, anim_def)
    }
}

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct IfPmC {
    cond_type: CondType, // 00
    zero4: u32,          // 04
    value: Bytes<4>,     // 08
    unk12: u32,          // 12
}
impl_as_bytes!(IfPmC, 16);

impl EventPm for If {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(IfPmC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("if size", size == IfPmC::SIZE, read.offset)?;
        let if_: IfPmC = read.read_struct()?;

        let cond_type = assert_that!("if cond", enum if_.cond_type, read.prev + 0)?;
        assert_that!("if field 12", if_.unk12 == 0, read.prev + 12)?;
        let value: [u8; 4] = if_.value.into_inner();

        let condition = assert_cond(cond_type, if_.zero4, value, read.prev)?;
        Ok(If { condition })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, _anim_def: &AnimDef) -> Result<()> {
        let (cond_type, zero4, value) = make_cond(&self.condition);

        let if_ = IfPmC {
            cond_type: cond_type.maybe(),
            zero4,
            value: value.into(),
            unk12: 0,
        };
        write.write_struct(&if_)?;
        Ok(())
    }
}

impl EventPm for Elseif {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(IfPmC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("else if size", size == IfPmC::SIZE, read.offset)?;
        let if_: IfPmC = read.read_struct()?;

        let cond_type = assert_that!("else if cond", enum if_.cond_type, read.prev + 0)?;
        assert_that!("else if field 12", if_.unk12 == 0, read.prev + 12)?;
        let value: [u8; 4] = if_.value.into();

        let condition = assert_cond(cond_type, if_.zero4, value, read.prev)?;
        Ok(Self { condition })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, _anim_def: &AnimDef) -> Result<()> {
        let (cond_type, zero4, value) = make_cond(&self.condition);

        let if_ = IfPmC {
            cond_type: cond_type.maybe(),
            zero4,
            value: value.into(),
            unk12: 0,
        };
        write.write_struct(&if_)?;
        Ok(())
    }
}

impl EventAll for Else {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(0)
    }

    fn read(read: &mut CountingReader<impl Read>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("else size", size == 0, read.offset)?;
        Ok(Self {})
    }

    fn write(&self, _write: &mut CountingWriter<impl Write>, _anim_def: &AnimDef) -> Result<()> {
        Ok(())
    }
}

impl EventAll for Endif {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(0)
    }

    fn read(read: &mut CountingReader<impl Read>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("end if size", size == 0, read.offset)?;
        Ok(Self {})
    }

    fn write(&self, _write: &mut CountingWriter<impl Write>, _anim_def: &AnimDef) -> Result<()> {
        Ok(())
    }
}
