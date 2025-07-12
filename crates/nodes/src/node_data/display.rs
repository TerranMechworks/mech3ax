use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::Color;
use mech3ax_api_types::nodes::Display;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{Result, assert_that};
use mech3ax_types::{AsBytes as _, impl_as_bytes};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct DisplayC {
    origin_x: u32,
    origin_y: u32,
    resolution_x: u32,
    resolution_y: u32,
    clear_color: Color,
}
impl_as_bytes!(DisplayC, 28);

pub(crate) fn read(read: &mut CountingReader<impl Read>, data_ptr: u32) -> Result<Display> {
    let display: DisplayC = read.read_struct()?;

    assert_that!("display origin x", display.origin_x == 0, read.prev + 0)?;
    assert_that!("display origin y", display.origin_y == 0, read.prev + 4)?;
    // assert_that!(
    //     "display resolution x",
    //     display.resolution_x == 640,
    //     read.prev + 8
    // )?;
    // // rc = 400, mw = 400, pm = 400, cs = 480
    // assert_that!(
    //     "display resolution y",
    //     display.resolution_y in [400, 480],
    //     read.prev + 12
    // )?;
    assert_that!("display clear color r", 0.0 <= display.clear_color.r <= 1.0, read.prev + 16)?;
    assert_that!("display clear color g", 0.0 <= display.clear_color.g <= 1.0, read.prev + 20)?;
    assert_that!("display clear color b", 0.0 <= display.clear_color.b <= 1.0, read.prev + 24)?;

    Ok(Display {
        resolution_x: display.resolution_x,
        resolution_y: display.resolution_y,
        clear_color: display.clear_color,
        data_ptr,
    })
}

pub(crate) fn write(write: &mut CountingWriter<impl Write>, display: &Display) -> Result<()> {
    let display = DisplayC {
        origin_x: 0,
        origin_y: 0,
        resolution_x: display.resolution_x,
        resolution_y: display.resolution_y,
        clear_color: display.clear_color,
    };
    write.write_struct(&display)?;
    Ok(())
}

pub(crate) fn size() -> u32 {
    DisplayC::SIZE
}
