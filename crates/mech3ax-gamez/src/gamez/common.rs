use log::{debug, trace};
use mech3ax_api_types::{static_assert_size, ReprSize as _};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use std::io::{Read, Write};

pub const SIGNATURE: u32 = 0x02971222;

pub const VERSION_RC: u32 = 15;
pub const VERSION_MW: u32 = 27;
pub const VERSION_PM: u32 = 41;
pub const VERSION_CS: u32 = 42;

#[derive(Debug)]
#[repr(C)]
struct MeshesInfoC {
    array_size: i32, // 00
    count: i32,      // 04
    index_max: i32,  // 08
}
static_assert_size!(MeshesInfoC, 12);
pub const MESHES_INFO_C_SIZE: u32 = MeshesInfoC::SIZE;

pub fn read_meshes_info_sequential(read: &mut CountingReader<impl Read>) -> Result<(i32, i32)> {
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
        info.index_max == info.count,
        read.prev + 8
    )?;

    Ok((info.array_size, info.count))
}

pub fn write_meshes_info_sequential(
    write: &mut CountingWriter<impl Write>,
    array_size: i32,
    count: i32,
) -> Result<()> {
    debug!(
        "Writing mesh info (rc, {}) at {}",
        MeshesInfoC::SIZE,
        write.offset
    );
    let info = MeshesInfoC {
        array_size,
        count,
        index_max: count,
    };
    trace!("{:#?}", info);
    write.write_struct(&info)?;
    Ok(())
}
