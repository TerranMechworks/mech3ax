use crate::flags::NodeBitFlags;
use crate::types::{NodeVariantMw, NodeVariantsMw, ZONE_DEFAULT};
use log::{debug, trace};
use mech3ax_api_types::gamez::{BoundingBox, Window};
use mech3ax_api_types::{static_assert_size, ReprSize as _};
use mech3ax_common::assert::assert_all_zero;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_debug::Zeros;
use std::io::{Read, Write};

#[derive(Debug)]
#[repr(C)]
struct WindowMwC {
    origin_x: u32,       // 000
    origin_y: u32,       // 004
    resolution_x: u32,   // 008
    resolution_y: u32,   // 012
    zero016: Zeros<212>, // 016
    buffer_index: i32,   // 228
    buffer_ptr: u32,     // 232
    zero236: u32,        // 236
    zero240: u32,        // 240
    zero244: u32,        // 244
}
static_assert_size!(WindowMwC, 248);

const WINDOW_NAME: &str = "window1";

pub fn assert_variants(node: NodeVariantsMw, offset: u32) -> Result<NodeVariantMw> {
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
    assert_that!(
        "window bbox 1",
        node.unk116 == BoundingBox::EMPTY,
        offset + 116
    )?;
    assert_that!(
        "window bbox 2",
        node.unk140 == BoundingBox::EMPTY,
        offset + 140
    )?;
    assert_that!(
        "window bbox 3",
        node.unk164 == BoundingBox::EMPTY,
        offset + 164
    )?;
    assert_that!("window field 196", node.unk196 == 0, offset + 196)?;
    Ok(NodeVariantMw::Window(node.data_ptr))
}

pub fn read(read: &mut CountingReader<impl Read>, data_ptr: u32, index: usize) -> Result<Window> {
    debug!(
        "Reading window node data {} (mw, {}) at {}",
        index,
        WindowMwC::SIZE,
        read.offset
    );
    let window: WindowMwC = read.read_struct()?;
    trace!("{:#?}", window);

    assert_that!("origin x", window.origin_x == 0, read.prev + 0)?;
    assert_that!("origin y", window.origin_y == 0, read.prev + 4)?;
    assert_that!("resolution x", window.resolution_x == 320, read.prev + 8)?;
    assert_that!("resolution y", window.resolution_y == 200, read.prev + 12)?;
    assert_all_zero("field 016", read.prev + 16, &window.zero016.0)?;
    assert_that!("buffer index", window.buffer_index == -1, read.prev + 228)?;
    assert_that!("buffer ptr", window.buffer_ptr == 0, read.prev + 232)?;
    assert_that!("zero236", window.zero236 == 0, read.prev + 236)?;
    assert_that!("zero240", window.zero240 == 0, read.prev + 240)?;
    assert_that!("zero244", window.zero244 == 0, read.prev + 244)?;

    Ok(Window {
        name: WINDOW_NAME.to_owned(),
        resolution_x: window.resolution_x,
        resolution_y: window.resolution_y,
        data_ptr,
    })
}

pub fn make_variants(window: &Window) -> NodeVariantsMw {
    NodeVariantsMw {
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
        unk116: BoundingBox::EMPTY,
        unk140: BoundingBox::EMPTY,
        unk164: BoundingBox::EMPTY,
        unk196: 0,
    }
}

pub fn write(write: &mut CountingWriter<impl Write>, window: &Window, index: usize) -> Result<()> {
    debug!(
        "Writing window node data {} (mw, {}) at {}",
        index,
        WindowMwC::SIZE,
        write.offset
    );
    let window = WindowMwC {
        origin_x: 0,
        origin_y: 0,
        resolution_x: window.resolution_x,
        resolution_y: window.resolution_y,
        zero016: Zeros::new(),
        buffer_index: -1,
        buffer_ptr: 0,
        zero236: 0,
        zero240: 0,
        zero244: 0,
    };
    trace!("{:#?}", window);
    write.write_struct(&window)?;
    Ok(())
}

pub fn size() -> u32 {
    WindowMwC::SIZE
}
