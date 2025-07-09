use mech3ax_api_types::{Count, Index};
use mech3ax_common::check::amend_err;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::Result;
use std::io::{Read, Write};

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
    count: Count,
    mut err: F,
    file: &'static str,
    line: u32,
) -> Result<Vec<Index>>
where
    F: FnMut(i16, i16) -> String,
{
    let count = count.to_i16();
    (0..count)
        .map(|index| {
            let value = read.read_i32()?;
            Index::check_i32(value).map_err(|msg| {
                let name = err(index, count);
                amend_err(msg, &name, read.prev, file, line).into()
            })
        })
        .collect::<Result<Vec<Index>>>()
}

pub(crate) fn write_node_indices(
    write: &mut CountingWriter<impl Write>,
    indices: &[Index],
) -> Result<()> {
    for index in indices.iter().copied() {
        let value = index.to_i32();
        write.write_i32(value)?;
    }
    Ok(())
}
