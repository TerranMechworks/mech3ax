use crate::nodes::check::color;
use mech3ax_api_types::gamez::nodes::Display;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{chk, Result};
use std::io::Read;

pub(crate) fn read(read: &mut CountingReader<impl Read>) -> Result<Display> {
    let display: Display = read.read_struct()?;
    assert_display(display, read.prev)
}

fn assert_display(display: Display, offset: usize) -> Result<Display> {
    chk!(offset, color(display.clear_color.r))?;
    chk!(offset, color(display.clear_color.g))?;
    chk!(offset, color(display.clear_color.b))?;
    Ok(display)
}
