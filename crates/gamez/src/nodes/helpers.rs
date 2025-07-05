use mech3ax_common::check::amend_err;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::Result;
use std::io::Read;

macro_rules! read_node_indices {
    ($read:expr, $count:expr, $err:expr) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $crate::nodes::helpers::_read_node_indices($read, $count, $err, FILE, LINE)
    }};
}
pub(crate) use read_node_indices;

pub(crate) fn _read_node_indices<F>(
    read: &mut CountingReader<impl Read>,
    count: u16,
    mut err: F,
    file: &str,
    line: u32,
) -> Result<Vec<u16>>
where
    F: FnMut(u16, u16) -> String,
{
    (0..count)
        .map(|index| {
            let value = read.read_i32()?;
            crate::nodes::check::node_index2(value).map_err(|msg| {
                let name = err(index, count);
                // TODO
                amend_err(msg, &name, read.prev, file, line).into()
            })
        })
        .collect::<Result<Vec<u16>>>()
}
