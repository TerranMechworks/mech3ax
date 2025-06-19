use log::trace;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use std::io::{Read, Result, Write};

pub fn read_child_indices(read: &mut CountingReader<impl Read>, count: u32) -> Result<Vec<u32>> {
    if count > 0 {
        trace!("Processing {} child indices at {}", count, read.offset);
        (0..count)
            .map(|_| read.read_u32())
            .collect::<std::io::Result<Vec<_>>>()
    } else {
        Ok(Vec::new())
    }
}

pub fn write_child_indices(write: &mut CountingWriter<impl Write>, indices: &[u32]) -> Result<()> {
    if !indices.is_empty() {
        trace!(
            "Processing {} child indices at {}",
            indices.len(),
            write.offset
        );
        for child in indices.iter().copied() {
            write.write_u32(child)?;
        }
    }
    Ok(())
}
