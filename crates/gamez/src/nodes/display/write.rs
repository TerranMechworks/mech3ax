use mech3ax_api_types::gamez::nodes::Display;
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::Result;
use std::io::Write;

pub(crate) fn write(write: &mut CountingWriter<impl Write>, display: &Display) -> Result<()> {
    write.write_struct(display)?;
    Ok(())
}
