use super::flags::NodeBitFlags;
use super::types::{NodeVariant, NodeVariants, Window, BLOCK_EMPTY, ZONE_DEFAULT};
use crate::assert::assert_all_zero;
use crate::io_ext::{CountingReader, WriteHelper};
use crate::size::ReprSize;
use crate::{assert_that, static_assert_size, Result};
use std::io::{Read, Write};

#[repr(C)]
struct WindowC {
    origin_x: u32,          // 000
    origin_y: u32,          // 004
    resolution: (u32, u32), // 008
    zero016: [u8; 212],
    buffer_index: i32, // 228
    buffer_ptr: u32,   // 232
    zero236: u32,
    zero240: u32,
    zero244: u32,
}
static_assert_size!(WindowC, 248);

const WINDOW_NAME: &str = "window1";

pub fn assert_variants(node: NodeVariants, offset: u32) -> Result<NodeVariant> {
    let name = &node.name;
    assert_that!("window name", name == WINDOW_NAME, offset + 0)?;
    assert_that!(
        "window flags",
        node.flags == NodeBitFlags::DEFAULT,
        offset + 36
    )?;
    assert_that!("window field 044", node.unk044 == 0, offset + 44)?;
    assert_that!("window zone id", node.zone_id == ZONE_DEFAULT, offset + 48)?;
    assert_that!("window data ptr", node.data_ptr != 0, offset + 56)?;
    assert_that!("window mesh index", node.mesh_index == -1, offset + 60)?;
    assert_that!(
        "window area partition",
        node.area_partition == None,
        offset + 76
    )?;
    assert_that!("window has parent", node.has_parent == false, offset + 84)?;
    // parent array ptr is already asserted
    assert_that!(
        "window children count",
        node.children_count == 0,
        offset + 92
    )?;
    // children array ptr is already asserted
    assert_that!("window block 1", node.unk116 == BLOCK_EMPTY, offset + 116)?;
    assert_that!("window block 2", node.unk140 == BLOCK_EMPTY, offset + 140)?;
    assert_that!("window block 3", node.unk164 == BLOCK_EMPTY, offset + 164)?;
    assert_that!("window field 196", node.unk196 == 0, offset + 196)?;
    Ok(NodeVariant::Window(node.data_ptr))
}

pub fn read<R>(read: &mut CountingReader<R>, data_ptr: u32) -> Result<Window>
where
    R: Read,
{
    let window: WindowC = read.read_struct()?;
    assert_that!("origin x", window.origin_x == 0, read.prev + 0)?;
    assert_that!("origin y", window.origin_y == 0, read.prev + 4)?;
    assert_that!("resolution", window.resolution == (320, 200), read.prev + 8)?;
    assert_all_zero("field 016", read.prev + 16, &window.zero016)?;
    assert_that!("buffer index", window.buffer_index == -1, read.prev + 228)?;
    assert_that!("buffer ptr", window.buffer_ptr == 0, read.prev + 232)?;
    assert_that!("zero236", window.zero236 == 0, read.prev + 236)?;
    assert_that!("zero240", window.zero240 == 0, read.prev + 240)?;
    assert_that!("zero244", window.zero244 == 0, read.prev + 244)?;

    Ok(Window {
        name: WINDOW_NAME.to_owned(),
        resolution: window.resolution,
        data_ptr,
    })
}

pub fn make_variants(window: &Window) -> NodeVariants {
    NodeVariants {
        name: WINDOW_NAME.to_owned(),
        flags: NodeBitFlags::DEFAULT,
        unk044: 0,
        zone_id: ZONE_DEFAULT,
        data_ptr: window.data_ptr,
        mesh_index: -1,
        area_partition: None,
        has_parent: false,
        parent_array_ptr: 0,
        children_count: 0,
        children_array_ptr: 0,
        unk116: BLOCK_EMPTY,
        unk140: BLOCK_EMPTY,
        unk164: BLOCK_EMPTY,
        unk196: 0,
    }
}

pub fn write<W>(write: &mut W, window: &Window) -> Result<()>
where
    W: Write,
{
    write.write_struct(&WindowC {
        origin_x: 0,
        origin_y: 0,
        resolution: window.resolution,
        zero016: [0; 212],
        buffer_index: -1,
        buffer_ptr: 0,
        zero236: 0,
        zero240: 0,
        zero244: 0,
    })?;
    Ok(())
}

pub fn size() -> u32 {
    WindowC::SIZE
}
