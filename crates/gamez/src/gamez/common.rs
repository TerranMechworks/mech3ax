use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{impl_as_bytes, AsBytes as _};
use std::io::{Read, Write};
use std::ops::Range;

pub(crate) const SIGNATURE: u32 = 0x02971222;

pub(crate) const VERSION_RC: u32 = 15;
pub(crate) const VERSION_MW: u32 = 27;
pub(crate) const VERSION_PM: u32 = 41;
#[expect(dead_code)]
pub(crate) const VERSION_CS: u32 = 42;

// we'll never know why???
pub(crate) const NODE_INDEX_INVALID: i32 = 0x00FFFFFF;

pub(crate) const NODE_INDEX_TOP_MASK: u32 = 0xFF000000;
pub(crate) const NODE_INDEX_BOT_MASK: u32 = 0x00FFFFFF;
pub(crate) const NODE_INDEX_TOP: u32 = 0x02000000;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
pub(crate) struct ModelArrayC {
    pub(crate) array_size: i32, // 00
    pub(crate) count: i32,      // 04
    pub(crate) last_index: i32, // 08
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
    pub(crate) count: i32,
    pub(crate) array_size: i32,
    #[allow(dead_code)]
    pub(crate) last_index: i32,
}

impl ModelArray {
    pub(crate) fn valid(&self) -> Range<i32> {
        0..self.count
    }

    pub(crate) fn zeros(&self) -> ModelIndexIter {
        ModelIndexIter(self.count..self.array_size)
    }
}

pub(crate) fn read_model_array_sequential(
    read: &mut CountingReader<impl Read>,
) -> Result<ModelArray> {
    let info: ModelArrayC = read.read_struct()?;

    assert_that!("model array size", 1 <= info.array_size <= i32::MAX - 1, read.prev + 0)?;
    assert_that!("model count", info.count < info.array_size, read.prev + 4)?;
    assert_that!(
        "model index max",
        info.last_index == info.count,
        read.prev + 8
    )?;

    Ok(ModelArray {
        array_size: info.array_size,
        count: info.count,
        last_index: info.last_index,
    })
}

pub(crate) fn write_model_array_sequential(
    write: &mut CountingWriter<impl Write>,
    array_size: i32,
    count: i32,
) -> Result<ModelIndexIter> {
    let info = ModelArrayC {
        array_size,
        count,
        last_index: count,
    };
    write.write_struct(&info)?;
    Ok(ModelIndexIter(count..array_size))
}
