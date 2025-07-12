use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::nodes::Window;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{Result, assert_that};
use mech3ax_types::{AsBytes as _, Zeros, impl_as_bytes};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Default)]
#[repr(C)]
struct WindowC {
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
impl_as_bytes!(WindowC, 248);

pub(crate) fn read(read: &mut CountingReader<impl Read>, data_ptr: u32) -> Result<Window> {
    let window: WindowC = read.read_struct()?;

    assert_that!("window origin x", window.origin_x == 0, read.prev + 0)?;
    assert_that!("window origin y", window.origin_y == 0, read.prev + 4)?;
    // // rc = 320, mw = 320, pm = 320, cs = 640/96
    // assert_that!(
    //     "window resolution x",
    //     window.resolution_x in [96, 320, 640],
    //     read.prev + 8
    // )?;
    // // rc = 200, mw =  200, pm = 200, cs = 480/96
    // assert_that!(
    //     "window resolution y",
    //     window.resolution_y in [96, 200, 480],
    //     read.prev + 12
    // )?;
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

    Ok(Window {
        resolution_x: window.resolution_x,
        resolution_y: window.resolution_y,
        data_ptr,
    })
}

pub(crate) fn write(write: &mut CountingWriter<impl Write>, window: &Window) -> Result<()> {
    let window = WindowC {
        resolution_x: window.resolution_x,
        resolution_y: window.resolution_y,
        buffer_index: -1,
        ..Default::default()
    };
    write.write_struct(&window)?;
    Ok(())
}

pub(crate) fn size() -> u32 {
    WindowC::SIZE
}
