use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::{Count, Count32};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{chk, Result};
use mech3ax_types::{impl_as_bytes, AsBytes as _, Offsets};
use std::io::{Read, Write};
use std::ops::Range;

pub(crate) const SIGNATURE: u32 = 0x02971222;

pub(crate) const VERSION_RC: u32 = 15;
pub(crate) const VERSION_MW: u32 = 27;
pub(crate) const VERSION_PM: u32 = 41;
#[expect(dead_code)]
pub(crate) const VERSION_CS: u32 = 42;

pub(crate) fn texture_count(value: Count32) -> Result<Count, String> {
    let v: i32 = value.value;
    if (0..4096).contains(&v) {
        value.check()
    } else {
        Err(format!("expected {} in 0..4096", v))
    }
}

// we'll never know why???
pub(crate) const NODE_INDEX_INVALID: i32 = 0x00FFFFFF;

pub(crate) const NODE_INDEX_TOP_MASK: u32 = 0xFF000000;
pub(crate) const NODE_INDEX_BOT_MASK: u32 = 0x00FFFFFF;
pub(crate) const NODE_INDEX_TOP: u32 = 0x02000000;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
pub(crate) struct ModelArrayC {
    pub(crate) array_size: Count32, // 00
    pub(crate) count: Count32,      // 04
    pub(crate) last_index: Count32, // 08
}
impl_as_bytes!(ModelArrayC, 12);
pub(crate) const MODEL_ARRAY_C_SIZE: u32 = ModelArrayC::SIZE;

#[derive(Debug)]
#[repr(transparent)]
pub(crate) struct ModelIndexIter(Range<i32>);

impl Iterator for ModelIndexIter {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.0.next()?;
        let mut expected_index = index + 1;
        if expected_index == self.0.end {
            expected_index = -1;
        }
        Some((index, expected_index))
    }
}

#[derive(Debug)]
pub(crate) struct ModelArray {
    pub(crate) count: Count,
    pub(crate) array_size: Count,
    #[allow(dead_code)]
    pub(crate) last_index: Count,
}

impl ModelArray {
    pub(crate) fn valid(&self) -> Range<i32> {
        0..self.count.to_i32()
    }

    pub(crate) fn zeros(&self) -> ModelIndexIter {
        ModelIndexIter(self.count.to_i32()..self.array_size.to_i32())
    }
}

pub(crate) fn read_model_array_sequential(
    read: &mut CountingReader<impl Read>,
) -> Result<ModelArray> {
    let info: ModelArrayC = read.read_struct()?;
    let offset = read.prev;

    let array_size = chk!(offset, ?info.array_size)?;
    let count = chk!(offset, ?info.count)?;
    let last_index = chk!(offset, ?info.last_index)?;

    chk!(offset, info.count < info.array_size)?;
    chk!(offset, info.last_index == info.count)?;

    Ok(ModelArray {
        array_size,
        count,
        last_index,
    })
}

pub(crate) fn write_model_array_sequential(
    write: &mut CountingWriter<impl Write>,
    array_size: Count,
    count: Count,
) -> Result<ModelIndexIter> {
    let info = ModelArrayC {
        array_size: array_size.maybe(),
        count: count.maybe(),
        last_index: count.maybe(),
    };
    write.write_struct(&info)?;
    Ok(ModelIndexIter(count.to_i32()..array_size.to_i32()))
}
