use super::{WindowC, WindowClearPolygonC};
use mech3ax_api_types::gamez::nodes::Window;
use mech3ax_common::Result;
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_types::Ptr;
use std::io::Write;

pub(crate) fn write(write: &mut CountingWriter<impl Write>, window: &Window) -> Result<()> {
    let window = WindowC {
        origin_x: window.origin_x,
        origin_y: window.origin_y,
        resolution_x: window.resolution_x,
        resolution_y: window.resolution_y,
        clear_polygon0: WindowClearPolygonC::EMPTY,
        clear_polygon1: WindowClearPolygonC::EMPTY,
        clear_polygon2: WindowClearPolygonC::EMPTY,
        clear_polygon3: WindowClearPolygonC::EMPTY,
        clear_polygon_count: 0,
        buffer_index: -1,
        buffer_surface_ptr: Ptr::NULL,
        buffer_width: 0,
        buffer_height: 0,
        buffer_bit_depth: 0,
    };
    write.write_struct(&window)?;
    Ok(())
}
