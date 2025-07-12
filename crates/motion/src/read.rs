use super::{MotionFlags, MotionHeaderC, VERSION};
use log::trace;
use mech3ax_api_types::motion::{Motion, MotionFrame, MotionPart};
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{Result, assert_that};
use mech3ax_types::Maybe;
use std::io::Read;

pub fn read_motion(read: &mut CountingReader<impl Read>) -> Result<Motion> {
    let header: MotionHeaderC = read.read_struct()?;

    assert_that!("motion version", header.version == VERSION, read.prev)?;
    assert_that!("motion loop time", header.loop_time > 0.0, read.prev + 4)?;
    assert_that!("motion frame count", header.frame_count > 0, read.prev + 8)?;
    assert_that!("motion field 16", header.unk16 == -1.0, read.prev + 16)?;
    assert_that!("motion field 20", header.unk20 == 1.0, read.prev + 20)?;

    let frame_count = header.frame_count;
    let parts = (0..header.part_count)
        .map(|index| {
            trace!("Reading motion part {}", index);
            let part_name = read.read_string()?;

            let flags = Maybe::new(read.read_u32()?);
            assert_that!(
                "motion part flags",
                flags == MotionFlags::DEFAULT.maybe(),
                read.prev
            )?;

            let mut translations = (0..=frame_count)
                .map(|_| read.read_struct_no_log())
                .collect::<std::io::Result<Vec<_>>>()?;

            // the first and last frames always match
            let first = translations.first().unwrap();
            let last = translations.last().unwrap();
            assert_that!(
                "motion part translation first/last",
                first == last,
                read.offset
            )?;
            translations.pop();

            let mut rotations = (0..=frame_count)
                .map(|_| read.read_struct_no_log())
                .collect::<std::io::Result<Vec<_>>>()?;

            // the first and last frames always match
            let first = rotations.first().unwrap();
            let last = rotations.last().unwrap();
            assert_that!(
                "motion part rotation first/last",
                first == last,
                read.offset
            )?;
            rotations.pop();

            let frames = translations
                .into_iter()
                .zip(rotations.into_iter())
                .map(|(translation, rotation)| MotionFrame {
                    translation,
                    rotation,
                })
                .collect();

            Ok(MotionPart {
                name: part_name,
                frames,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    read.assert_end()?;

    Ok(Motion {
        loop_time: header.loop_time,
        parts,
        frame_count,
    })
}
