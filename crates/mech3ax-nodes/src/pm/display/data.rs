use super::info::DISPLAY_NAME;
use log::{debug, trace};
use mech3ax_api_types::nodes::pm::Display;
use mech3ax_api_types::{static_assert_size, Color, ReprSize as _};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use std::io::{Read, Write};

#[derive(Debug)]
#[repr(C)]
struct DisplayPmC {
    origin_x: u32,
    origin_y: u32,
    resolution_x: u32,
    resolution_y: u32,
    clear_color: Color,
}
static_assert_size!(DisplayPmC, 28);

#[allow(clippy::excessive_precision)]
const CLEAR_COLOR: Color = Color {
    r: 0.3919999897480011,
    g: 0.3919999897480011,
    b: 1.0,
};

pub fn read(read: &mut CountingReader<impl Read>, data_ptr: u32, index: usize) -> Result<Display> {
    debug!(
        "Reading display node data {} (pm, {}) at {}",
        index,
        DisplayPmC::SIZE,
        read.offset
    );
    let display: DisplayPmC = read.read_struct()?;
    trace!("{:#?}", display);

    assert_that!("display origin x", display.origin_x == 0, read.prev + 0)?;
    assert_that!("display origin y", display.origin_y == 0, read.prev + 4)?;
    assert_that!(
        "display resolution x",
        display.resolution_x == 640,
        read.prev + 8
    )?;
    assert_that!(
        "display resolution y",
        display.resolution_y == 400,
        read.prev + 12
    )?;
    assert_that!(
        "display clear color",
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

pub fn write(
    write: &mut CountingWriter<impl Write>,
    display: &Display,
    index: usize,
) -> Result<()> {
    debug!(
        "Writing display node data {} (pm, {}) at {}",
        index,
        DisplayPmC::SIZE,
        write.offset
    );
    let display = DisplayPmC {
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
