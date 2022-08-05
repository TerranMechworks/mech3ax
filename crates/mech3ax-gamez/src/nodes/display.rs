use super::flags::NodeBitFlags;
use super::types::{NodeVariant, NodeVariants, ZONE_DEFAULT};
use mech3ax_api_types::{static_assert_size, Block, Color, Display, ReprSize as _};
use mech3ax_common::io_ext::{CountingReader, WriteHelper};
use mech3ax_common::{assert_that, Result};
use std::io::{Read, Write};

#[repr(C)]
struct DisplayC {
    origin_x: u32,
    origin_y: u32,
    resolution: (u32, u32),
    clear_color: Color,
}
static_assert_size!(DisplayC, 28);

#[allow(clippy::excessive_precision)]
const CLEAR_COLOR: Color = Color {
    r: 0.3919999897480011,
    g: 0.3919999897480011,
    b: 1.0,
};
const DISPLAY_NAME: &str = "display";

pub fn assert_variants(node: NodeVariants, offset: u32) -> Result<NodeVariant> {
    let name = &node.name;
    assert_that!("display name", name == DISPLAY_NAME, offset + 0)?;
    assert_that!(
        "display flags",
        node.flags == NodeBitFlags::DEFAULT,
        offset + 36
    )?;
    assert_that!("display field 044", node.unk044 == 0, offset + 44)?;
    assert_that!("display zone id", node.zone_id == ZONE_DEFAULT, offset + 48)?;
    assert_that!("display data ptr", node.data_ptr != 0, offset + 56)?;
    assert_that!("display mesh index", node.mesh_index == -1, offset + 60)?;
    assert_that!(
        "display area partition",
        node.area_partition == None,
        offset + 76
    )?;
    assert_that!("display has parent", node.has_parent == false, offset + 84)?;
    // parent array ptr is already asserted
    assert_that!(
        "display children count",
        node.children_count == 0,
        offset + 92
    )?;
    // children array ptr is already asserted
    assert_that!("display block 1", node.unk116 == Block::EMPTY, offset + 116)?;
    assert_that!("display block 2", node.unk140 == Block::EMPTY, offset + 140)?;
    assert_that!("display block 3", node.unk164 == Block::EMPTY, offset + 164)?;
    assert_that!("display field 196", node.unk196 == 0, offset + 196)?;
    Ok(NodeVariant::Display(node.data_ptr))
}

pub fn read<R>(read: &mut CountingReader<R>, data_ptr: u32) -> Result<Display>
where
    R: Read,
{
    let display: DisplayC = read.read_struct()?;
    assert_that!("origin x", display.origin_x == 0, read.prev + 0)?;
    assert_that!("origin y", display.origin_y == 0, read.prev + 4)?;
    assert_that!(
        "resolution",
        display.resolution == (640, 400),
        read.prev + 8
    )?;
    assert_that!(
        "clear color",
        display.clear_color == CLEAR_COLOR,
        read.prev + 16
    )?;

    Ok(Display {
        name: DISPLAY_NAME.to_owned(),
        resolution: display.resolution,
        clear_color: display.clear_color,
        data_ptr,
    })
}

pub fn make_variants(display: &Display) -> NodeVariants {
    NodeVariants {
        name: DISPLAY_NAME.to_owned(),
        flags: NodeBitFlags::DEFAULT,
        unk044: 0,
        zone_id: ZONE_DEFAULT,
        data_ptr: display.data_ptr,
        mesh_index: -1,
        area_partition: None,
        has_parent: false,
        parent_array_ptr: 0,
        children_count: 0,
        children_array_ptr: 0,
        unk116: Block::EMPTY,
        unk140: Block::EMPTY,
        unk164: Block::EMPTY,
        unk196: 0,
    }
}

pub fn write<W>(write: &mut W, display: &Display) -> Result<()>
where
    W: Write,
{
    write.write_struct(&DisplayC {
        origin_x: 0,
        origin_y: 0,
        resolution: display.resolution,
        clear_color: display.clear_color,
    })?;
    Ok(())
}

pub fn size() -> u32 {
    DisplayC::SIZE
}
