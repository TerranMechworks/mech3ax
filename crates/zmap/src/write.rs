use super::{MapHeaderC, MAP_VERSION};
use log::trace;
use mech3ax_api_types::zmap::{MapFeature, Zmap};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{assert_len, Result};
use std::io::Write;

fn write_map_feature(
    write: &mut CountingWriter<impl Write>,
    feature: &MapFeature,
    index: usize,
) -> Result<()> {
    trace!("Writing map feature {}", index);
    write.write_struct(&feature.color)?;
    let count = assert_len!(u32, feature.vertices.len(), "map feature vertices")?;
    write.write_u32(count)?;
    trace!("Writing {} map feature vertices", count);
    for vertex in &feature.vertices {
        write.write_struct(vertex)?;
    }
    trace!("Map feature objective: {}", feature.objective);
    write.write_i32(feature.objective)?;
    Ok(())
}

pub fn write_map(write: &mut CountingWriter<impl Write>, map: &Zmap) -> Result<()> {
    let header = MapHeaderC {
        version: MAP_VERSION,
        unk04: map.unk04,
        min: map.min,
        max: map.max,
    };
    write.write_struct(&header)?;

    for (index, feature) in map.features.iter().enumerate() {
        write_map_feature(write, feature, index)?;
    }
    Ok(())
}
