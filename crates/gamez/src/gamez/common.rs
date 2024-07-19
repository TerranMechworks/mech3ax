use bytemuck::{AnyBitPattern, NoUninit};
use log::{debug, trace};
use mech3ax_api_types::{static_assert_size, ReprSize as _};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use std::io::{Read, Write};
use std::ops::Range;

pub const SIGNATURE: u32 = 0x02971222;

pub const VERSION_RC: u32 = 15;
pub const VERSION_MW: u32 = 27;
pub const VERSION_PM: u32 = 41;
pub const VERSION_CS: u32 = 42;

// we'll never know why???
pub const NODE_INDEX_INVALID: u32 = 0x00FFFFFF;
pub const NODE_INDEX_TOP_MASK: u32 = 0xFF000000;
pub const NODE_INDEX_BOT_MASK: u32 = 0x00FFFFFF;
pub const NODE_INDEX_TOP: u32 = 0x02000000;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
pub struct MeshesInfoC {
    pub array_size: i32, // 00
    pub count: i32,      // 04
    pub last_index: i32, // 08
}
static_assert_size!(MeshesInfoC, 12);
pub const MESHES_INFO_C_SIZE: u32 = MeshesInfoC::SIZE;

#[derive(Debug)]
#[repr(transparent)]
pub struct MeshIndexIter(Range<i32>);

impl Iterator for MeshIndexIter {
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
pub struct MeshIndices {
    pub count: i32,
    pub array_size: i32,
    #[allow(dead_code)]
    pub last_index: i32,
}

impl MeshIndices {
    pub fn valid(&self) -> Range<i32> {
        0..self.count
    }

    pub fn zeros(&self) -> MeshIndexIter {
        MeshIndexIter(self.count..self.array_size)
    }
}

pub fn read_meshes_info_sequential(read: &mut CountingReader<impl Read>) -> Result<MeshIndices> {
    debug!(
        "Reading mesh info ({}) at {}",
        MeshesInfoC::SIZE,
        read.offset
    );
    let info: MeshesInfoC = read.read_struct()?;
    trace!("{:#?}", info);

    assert_that!("mesh array size", 1 <= info.array_size <= i32::MAX - 1, read.prev + 0)?;
    assert_that!("mesh count", info.count < info.array_size, read.prev + 4)?;
    assert_that!(
        "mesh index max",
        info.last_index == info.count,
        read.prev + 8
    )?;

    Ok(MeshIndices {
        array_size: info.array_size,
        count: info.count,
        last_index: info.last_index,
    })
}

impl MeshesInfoC {
    pub fn iter(&self) -> MeshIndexIter {
        MeshIndexIter(0..self.array_size)
    }
}

pub fn read_meshes_info_nonseq(read: &mut CountingReader<impl Read>) -> Result<MeshesInfoC> {
    debug!(
        "Reading mesh info ({}) at {}",
        MeshesInfoC::SIZE,
        read.offset
    );
    let info: MeshesInfoC = read.read_struct()?;
    trace!("{:#?}", info);
    assert_that!("mesh array size", 1 <= info.array_size <= i32::MAX - 1, read.prev + 0)?;
    Ok(info)
}

pub fn write_meshes_info_sequential(
    write: &mut CountingWriter<impl Write>,
    array_size: i32,
    count: i32,
) -> Result<MeshIndexIter> {
    debug!(
        "Writing mesh info (rc, {}) at {}",
        MeshesInfoC::SIZE,
        write.offset
    );
    let info = MeshesInfoC {
        array_size,
        count,
        last_index: count,
    };
    trace!("{:#?}", info);
    write.write_struct(&info)?;
    Ok(MeshIndexIter(count..array_size))
}

pub fn write_meshes_info_nonseq(
    write: &mut CountingWriter<impl Write>,
    array_size: i32,
    count: i32,
    last_index: i32,
) -> Result<MeshesInfoC> {
    debug!(
        "Writing mesh info (rc, {}) at {}",
        MeshesInfoC::SIZE,
        write.offset
    );
    assert_that!("mesh array size", 1 <= array_size <= i32::MAX - 1, write.offset)?;
    let info = MeshesInfoC {
        array_size,
        count,
        last_index,
    };
    trace!("{:#?}", info);
    write.write_struct(&info)?;
    Ok(info)
}
