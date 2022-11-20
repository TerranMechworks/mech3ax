use super::{MapHeaderC, MAP_VERSION};
use log::{debug, trace};
use mech3ax_api_types::zmap::{MapColor, MapFeature, MapRc, MapVertex};
use mech3ax_api_types::ReprSize as _;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{assert_that, Result};
use std::io::Read;

fn read_map_feature(
    read: &mut CountingReader<impl Read>,
    max_x: f32,
    max_y: f32,
    color: MapColor,
    _index: usize,
) -> Result<MapFeature> {
    let count = read.read_u32()?;

    debug!(
        "Reading {} x map feature vertices at {}",
        count, read.offset
    );
    let vertices = (0..count)
        .map(|_| {
            let v: MapVertex = read.read_struct()?;
            assert_that!("map vertex x", v.x <= max_x, read.prev + 0)?;
            // v.z
            assert_that!("map vertex y", v.y <= max_y, read.prev + 8)?;
            Ok(v)
        })
        .collect::<Result<Vec<MapVertex>>>()?;

    let tail = read.read_i32()?;

    Ok(MapFeature {
        color,
        vertices,
        objective: tail,
    })
}

pub fn read_map(read: &mut CountingReader<impl Read>) -> Result<MapRc> {
    debug!(
        "Reading map header ({}) at {}",
        MapHeaderC::SIZE,
        read.offset
    );
    let header: MapHeaderC = read.read_struct()?;
    trace!("{:#?}", header);

    assert_that!(
        "map header version",
        header.version == MAP_VERSION,
        read.prev + 0
    )?;
    assert_that!("map header unk04", 1 <= header.unk04 <= 31, read.prev + 4)?;
    assert_that!("map header zero08", header.zero08 == 0, read.prev + 8)?;
    assert_that!("map header zero12", header.zero12 == 0, read.prev + 12)?;
    assert_that!("map header zero16", header.zero16 == 0, read.prev + 16)?;
    assert_that!("map header zero24", header.zero24 == 0, read.prev + 24)?;

    // it's currently not know how the number of features is encoded
    let mut features = Vec::new();
    for index in 0.. {
        debug!("Reading map feature {} at {}", index, read.offset);
        match read.read_struct::<MapColor>() {
            Ok(color) => {
                let feature = read_map_feature(read, header.max_x, header.max_y, color, index)?;
                features.push(feature);
            }
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }
    }
    // let features = (0..7)
    //     .map(|index| read_map_feature(read, header.max_x, header.max_y, index))
    //     .collect::<Result<Vec<_>>>()?;
    // read.assert_end()?;

    Ok(MapRc {
        unk04: header.unk04,
        max_x: header.max_x,
        max_y: header.max_y,
        features,
    })
}
