use super::info::{SPYGLASS_NAME, WINDOW_NAME};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::nodes::cs::Window;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{impl_as_bytes, Zeros};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct WindowCsC {
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
impl_as_bytes!(WindowCsC, 248);

pub(crate) fn read(
    read: &mut CountingReader<impl Read>,
    data_ptr: u32,
    spyglass: bool,
) -> Result<Window> {
    let window: WindowCsC = read.read_struct()?;

    assert_that!("window origin x", window.origin_x == 0, read.prev + 0)?;
    assert_that!("window origin y", window.origin_y == 0, read.prev + 4)?;
    let resolution_x = if spyglass { 96 } else { 640 };
    let resolution_y = if spyglass { 96 } else { 480 };
    assert_that!(
        "window resolution x",
        window.resolution_x == resolution_x,
        read.prev + 8
    )?;
    assert_that!(
        "window resolution y",
        window.resolution_y == resolution_y,
        read.prev + 12
    )?;
    assert_that!("window field 016", zero window.zero016, read.prev + 16)?;
    assert_that!(
        "window buffer index",
        window.buffer_index == -1,
        read.prev + 228
    )?;
    assert_that!("window buffer ptr", window.buffer_ptr == 0, read.prev + 232)?;
    assert_that!("window zero236", window.zero236 == 0, read.prev + 236)?;
    assert_that!("window zero240", window.zero240 == 0, read.prev + 240)?;
    assert_that!("window zero244", window.zero244 == 0, read.prev + 244)?;

    let name = if spyglass { SPYGLASS_NAME } else { WINDOW_NAME };
    Ok(Window {
        name: name.to_string(),
        resolution_x: window.resolution_x,
        resolution_y: window.resolution_y,
        data_ptr,
    })
}

pub(crate) fn write(write: &mut CountingWriter<impl Write>, window: &Window) -> Result<()> {
    let window = WindowCsC {
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
    write.write_struct(&window)?;
    Ok(())
}
