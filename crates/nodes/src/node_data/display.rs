use bytemuck::{AnyBitPattern, NoUninit};
use log::{debug, trace};
use mech3ax_api_types::nodes::Display;
use mech3ax_api_types::Color;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{impl_as_bytes, AsBytes as _};
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

pub fn read(read: &mut CountingReader<impl Read>, data_ptr: u32, index: usize) -> Result<Display> {
    debug!(
        "Reading display node data {} ({}) at {}",
        index,
        DisplayC::SIZE,
        read.offset
    );
    let display: DisplayC = read.read_struct()?;
    trace!("{:#?}", display);

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

pub fn write(
    write: &mut CountingWriter<impl Write>,
    display: &Display,
    index: usize,
) -> Result<()> {
    debug!(
        "Writing display node data {} ({}) at {}",
        index,
        DisplayC::SIZE,
        write.offset
    );
    let display = DisplayC {
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
    DisplayC::SIZE
}
