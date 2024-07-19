use super::ScriptObject;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::{CallSequence, StopSequence};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::{impl_as_bytes, AsBytes as _};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::string::{str_from_c_padded, str_to_c_padded};
use mech3ax_common::{assert_that, Result};
use mech3ax_debug::Ascii;
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct SequenceC {
    name: Ascii<32>,
    sentinel: i32,
}
impl_as_bytes!(SequenceC, 36);

fn read_sequence(read: &mut CountingReader<impl Read>) -> Result<String> {
    let sequence: SequenceC = read.read_struct()?;
    let name = assert_utf8("sequence name", read.prev + 0, || {
        str_from_c_padded(&sequence.name)
    })?;
    assert_that!("sequence field 32", sequence.sentinel == -1, read.prev + 32)?;
    Ok(name)
}

fn write_sequence(write: &mut CountingWriter<impl Write>, name: &str) -> Result<()> {
    let mut fill = Ascii::zero();
    str_to_c_padded(name, &mut fill);
    write.write_struct(&SequenceC {
        name: fill,
        sentinel: -1,
    })?;
    Ok(())
}

impl ScriptObject for CallSequence {
    const INDEX: u8 = 22;
    const SIZE: u32 = SequenceC::SIZE;

    fn read(read: &mut CountingReader<impl Read>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("call sequence size", size == Self::SIZE, read.offset)?;
        let name = read_sequence(read)?;
        Ok(Self { name })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, _anim_def: &AnimDef) -> Result<()> {
        write_sequence(write, &self.name)
    }
}

impl ScriptObject for StopSequence {
    const INDEX: u8 = 23;
    const SIZE: u32 = SequenceC::SIZE;

    fn read(read: &mut CountingReader<impl Read>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("stop sequence size", size == Self::SIZE, read.offset)?;
        let name = read_sequence(read)?;
        Ok(Self { name })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, _anim_def: &AnimDef) -> Result<()> {
        write_sequence(write, &self.name)
    }
}
