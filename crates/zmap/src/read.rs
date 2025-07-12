use super::{MAP_VERSION, MapHeaderC};
use log::trace;
use mech3ax_api_types::Vec3;
use mech3ax_api_types::zmap::{MapColor, MapFeature, Zmap};
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{Result, assert_that};
use std::io::Read;

fn read_map_feature(
    read: &mut CountingReader<impl Read>,
    color: MapColor,
    _index: usize,
) -> Result<MapFeature> {
    let count = read.read_u32()?;

    trace!("Reading {} map feature vertices", count);
    let vertices = (0..count)
        .map(|_| Ok(read.read_struct()?))
        .collect::<Result<Vec<Vec3>>>()?;

    let objective = read.read_i32()?;
    trace!("Map feature objective: {}", objective);

    Ok(MapFeature {
        color,
        vertices,
        objective,
    })
}

pub fn read_map(read: &mut CountingReader<impl Read>) -> Result<Zmap> {
    let header: MapHeaderC = read.read_struct()?;

    assert_that!(
        "map header version",
        header.version == MAP_VERSION,
        read.prev + 0
    )?;
    assert_that!("map header unk04", 1 <= header.unk04 <= 31, read.prev + 4)?;

    // it's currently not know how the number of features is encoded
    let mut features = Vec::new();
    for index in 0.. {
        trace!("Reading map feature {}", index);
        match read.read_struct::<MapColor>() {
            Ok(color) => {
                let feature = read_map_feature(read, color, index)?;
                features.push(feature);
            }
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                trace!("End of file");
                break;
            }
            Err(e) => return Err(e.into()),
        }
    }

    Ok(Zmap {
        unk04: header.unk04,
        min: header.min,
        max: header.max,
        features,
    })
}
