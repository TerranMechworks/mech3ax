use super::EventAll;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::{CallSequence, StopSequence};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{impl_as_bytes, AsBytes as _, Ascii};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct SequenceC {
    name: Ascii<32>,
    index: i32,
}
impl_as_bytes!(SequenceC, 36);

fn read_sequence(read: &mut CountingReader<impl Read>) -> Result<String> {
    let sequence: SequenceC = read.read_struct()?;

    let name = assert_utf8("sequence name", read.prev + 0, || {
        sequence.name.to_str_padded()
    })?;
    // we are currently reading sequences (or a reset state), so the sequence
    // name cannot be asserted at this point.
    assert_that!("sequence index", sequence.index == -1, read.prev + 32)?;
    Ok(name)
}

fn write_sequence(write: &mut CountingWriter<impl Write>, name: &str) -> Result<()> {
    let name = Ascii::from_str_padded(name);
    let sequence = SequenceC { name, index: -1 };
    write.write_struct(&sequence)?;
    Ok(())
}

impl EventAll for CallSequence {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(SequenceC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("call sequence size", size == SequenceC::SIZE, read.offset)?;
        let name = read_sequence(read)?;
        Ok(Self { name })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, _anim_def: &AnimDef) -> Result<()> {
        write_sequence(write, &self.name)
    }
}

impl EventAll for StopSequence {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(SequenceC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("stop sequence size", size == SequenceC::SIZE, read.offset)?;
        let name = read_sequence(read)?;
        Ok(Self { name })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, _anim_def: &AnimDef) -> Result<()> {
        write_sequence(write, &self.name)
    }
}
