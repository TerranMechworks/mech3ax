use super::ScriptObject;
use crate::AnimDef;
use mech3ax_api_types::{static_assert_size, ReprSize as _};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, WriteHelper};
use mech3ax_common::string::{str_from_c_padded, str_to_c_padded};
use mech3ax_common::{assert_that, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[repr(C)]
struct SequenceC {
    name: [u8; 32],
    sentinel: i32,
}
static_assert_size!(SequenceC, 36);

fn read_sequence<R: Read>(read: &mut CountingReader<R>) -> Result<String> {
    let sequence: SequenceC = read.read_struct()?;
    let name = assert_utf8("sequence name", read.prev + 0, || {
        str_from_c_padded(&sequence.name)
    })?;
    assert_that!("sequence field 32", sequence.sentinel == -1, read.prev + 32)?;
    Ok(name)
}

fn write_sequence<W: Write>(write: &mut W, name: &str) -> Result<()> {
    let mut fill = [0; 32];
    str_to_c_padded(name, &mut fill);
    write.write_struct(&SequenceC {
        name: fill,
        sentinel: -1,
    })?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CallSequence {
    pub name: String,
}

impl ScriptObject for CallSequence {
    const INDEX: u8 = 22;
    const SIZE: u32 = SequenceC::SIZE;

    fn read<R: Read>(read: &mut CountingReader<R>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("call sequence size", size == Self::SIZE, read.offset)?;
        let name = read_sequence(read)?;
        Ok(Self { name })
    }

    fn write<W: Write>(&self, write: &mut W, _anim_def: &AnimDef) -> Result<()> {
        write_sequence(write, &self.name)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StopSequence {
    pub name: String,
}

impl ScriptObject for StopSequence {
    const INDEX: u8 = 23;
    const SIZE: u32 = SequenceC::SIZE;

    fn read<R: Read>(read: &mut CountingReader<R>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("stop sequence size", size == Self::SIZE, read.offset)?;
        let name = read_sequence(read)?;
        Ok(Self { name })
    }

    fn write<W: Write>(&self, write: &mut W, _anim_def: &AnimDef) -> Result<()> {
        write_sequence(write, &self.name)
    }
}
