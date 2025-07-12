use super::{WindowC, WindowClearPolygonC};
use mech3ax_api_types::gamez::nodes::Window;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{Result, chk};
use mech3ax_types::Ptr;
use std::io::Read;

pub(crate) fn read(read: &mut CountingReader<impl Read>) -> Result<Window> {
    let window: WindowC = read.read_struct()?;
    assert_window(&window, read.prev)
}

fn assert_window(window: &WindowC, offset: usize) -> Result<Window> {
    chk!(offset, window.clear_polygon_count == 0)?;
    chk!(offset, window.clear_polygon0 == WindowClearPolygonC::EMPTY)?;
    chk!(offset, window.clear_polygon1 == WindowClearPolygonC::EMPTY)?;
    chk!(offset, window.clear_polygon2 == WindowClearPolygonC::EMPTY)?;
    chk!(offset, window.clear_polygon3 == WindowClearPolygonC::EMPTY)?;
    chk!(offset, window.buffer_index == -1)?;
    chk!(offset, window.buffer_surface_ptr == Ptr::NULL)?;
    chk!(offset, window.buffer_width == 0)?;
    chk!(offset, window.buffer_height == 0)?;
    chk!(offset, window.buffer_bit_depth == 0)?;

    Ok(Window {
        origin_x: window.origin_x,
        origin_y: window.origin_y,
        resolution_x: window.resolution_x,
        resolution_y: window.resolution_y,
    })
}
