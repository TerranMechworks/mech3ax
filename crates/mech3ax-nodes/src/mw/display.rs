use crate::flags::NodeBitFlags;
use crate::types::{NodeVariantMw, NodeVariantsMw, ZONE_DEFAULT};
use log::{debug, trace};
use mech3ax_api_types::nodes::mw::Display;
use mech3ax_api_types::nodes::BoundingBox;
use mech3ax_api_types::{static_assert_size, Color, ReprSize as _};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use std::io::{Read, Write};

#[derive(Debug)]
#[repr(C)]
struct DisplayMwC {
    origin_x: u32,
    origin_y: u32,
    resolution_x: u32,
    resolution_y: u32,
    clear_color: Color,
}
static_assert_size!(DisplayMwC, 28);

#[allow(clippy::excessive_precision)]
const CLEAR_COLOR: Color = Color {
    r: 0.3919999897480011,
    g: 0.3919999897480011,
    b: 1.0,
};
const DISPLAY_NAME: &str = "display";

pub fn assert_variants(node: NodeVariantsMw, offset: u32) -> Result<NodeVariantMw> {
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
    assert_that!(
        "display bbox 1",
        node.unk116 == BoundingBox::EMPTY,
        offset + 116
    )?;
    assert_that!(
        "display bbox 2",
        node.unk140 == BoundingBox::EMPTY,
        offset + 140
    )?;
    assert_that!(
        "display bbox 3",
        node.unk164 == BoundingBox::EMPTY,
        offset + 164
    )?;
    assert_that!("display field 196", node.unk196 == 0, offset + 196)?;
    Ok(NodeVariantMw::Display(node.data_ptr))
}

pub fn read(read: &mut CountingReader<impl Read>, data_ptr: u32, index: usize) -> Result<Display> {
    debug!(
        "Reading display node data {} (mw, {}) at {}",
        index,
        DisplayMwC::SIZE,
        read.offset
    );
    let display: DisplayMwC = read.read_struct()?;
    trace!("{:#?}", display);

    assert_that!("origin x", display.origin_x == 0, read.prev + 0)?;
    assert_that!("origin y", display.origin_y == 0, read.prev + 4)?;
    assert_that!("resolution x", display.resolution_x == 640, read.prev + 8)?;
    assert_that!("resolution y", display.resolution_y == 400, read.prev + 12)?;
    assert_that!(
        "clear color",
        display.clear_color == CLEAR_COLOR,
        read.prev + 16
    )?;

    Ok(Display {
        name: DISPLAY_NAME.to_owned(),
        resolution_x: display.resolution_x,
        resolution_y: display.resolution_y,
        clear_color: display.clear_color,
        data_ptr,
    })
}

pub fn make_variants(display: &Display) -> NodeVariantsMw {
    NodeVariantsMw {
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
        unk116: BoundingBox::EMPTY,
        unk140: BoundingBox::EMPTY,
        unk164: BoundingBox::EMPTY,
        unk196: 0,
    }
}

pub fn write(
    write: &mut CountingWriter<impl Write>,
    display: &Display,
    index: usize,
) -> Result<()> {
    debug!(
        "Writing display node data {} (mw, {}) at {}",
        index,
        DisplayMwC::SIZE,
        write.offset
    );
    let display = DisplayMwC {
        origin_x: 0,
        origin_y: 0,
        resolution_x: display.resolution_x,
        resolution_y: display.resolution_y,
        clear_color: display.clear_color,
    };
    trace!("{:#?}", display);
    write.write_struct(&display)?;
    Ok(())
}

pub fn size() -> u32 {
    DisplayMwC::SIZE
}
