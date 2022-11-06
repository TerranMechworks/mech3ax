use super::{MapHeaderC, MAP_VERSION};
use mech3ax_api_types::{MapChunk, MapRc};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{assert_len, Result};
use std::io::Write;

fn write_map_chunk(write: &mut CountingWriter<impl Write>, chunk: &MapChunk) -> Result<()> {
    let flags = [chunk.flag1, chunk.flag2, chunk.flag3];
    write.write_all(&flags)?;
    let count = assert_len!(u32, chunk.vertices.len(), "map chunk vertices")?;
    write.write_u32(count)?;
    for vertex in &chunk.vertices {
        write.write_struct(vertex)?;
    }
    write.write_i32(chunk.tail)?;
    Ok(())
}

pub fn write_map(write: &mut CountingWriter<impl Write>, map: &MapRc) -> Result<()> {
    let header = MapHeaderC {
        version: MAP_VERSION,
        unk04: map.unk04,
        zero08: 0,
        zero12: 0,
        zero16: 0,
        max_x: map.max_x,
        zero24: 0,
        max_y: map.max_y,
    };
    write.write_struct(&header)?;
    for chunk in &map.chunks {
        write_map_chunk(write, chunk)?;
    }
    Ok(())
}
