use super::{MapHeaderC, MAP_VERSION};
use log::{debug, trace};
use mech3ax_api_types::zmap::{MapFeature, Zmap};
use mech3ax_api_types::AsBytes as _;
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{assert_len, Result};
use std::io::Write;

fn write_map_feature(
    write: &mut CountingWriter<impl Write>,
    feature: &MapFeature,
    index: usize,
) -> Result<()> {
    debug!("Writing map feature {} at {}", index, write.offset);
    write.write_struct(&feature.color)?;
    let count = assert_len!(u32, feature.vertices.len(), "map feature vertices")?;
    write.write_u32(count)?;
    debug!(
        "Writing {} x map feature vertices at {}",
        count, write.offset
    );
    for vertex in &feature.vertices {
        write.write_struct(vertex)?;
    }
    write.write_i32(feature.objective)?;
    Ok(())
}

pub fn write_map(write: &mut CountingWriter<impl Write>, map: &Zmap) -> Result<()> {
    debug!(
        "Writing map header ({}) at {}",
        MapHeaderC::SIZE,
        write.offset
    );
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
    trace!("{:#?}", header);
    write.write_struct(&header)?;

    for (index, feature) in map.features.iter().enumerate() {
        write_map_feature(write, feature, index)?;
    }
    Ok(())
}
