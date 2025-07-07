use super::DisplayC;
use mech3ax_api_types::gamez::nodes::Display;
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::Result;
use std::io::Write;

pub(crate) fn write(write: &mut CountingWriter<impl Write>, display: &Display) -> Result<()> {
    let display = DisplayC {
        origin_x: display.origin_x,
        origin_y: display.origin_y,
        resolution_x: display.resolution_x,
        resolution_y: display.resolution_y,
        clear_color: display.clear_color,
    };
    write.write_struct(&display)?;
    Ok(())
}
