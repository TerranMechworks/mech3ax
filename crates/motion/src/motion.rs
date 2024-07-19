use bytemuck::{AnyBitPattern, NoUninit};
use log::{debug, trace};
use mech3ax_api_types::motion::{Motion, MotionFrame, MotionPart};
use mech3ax_api_types::{impl_as_bytes, AsBytes as _};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, assert_with_msg, Result};
use std::io::{Read, Write};

const VERSION: u32 = 4;
const FLAG_TRANSLATION: u32 = 1 << 3;
const FLAG_ROTATION: u32 = 1 << 2;
#[allow(unused)]
const FLAG_SCALING: u32 = 1 << 1; // never in motion.zbd
const FLAGS: u32 = FLAG_TRANSLATION | FLAG_ROTATION;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct MotionHeader {
    version: u32,     // 00
    loop_time: f32,   // 04
    frame_count: u32, // 08
    part_count: u32,  // 12
    unk16: f32,       // 16
    unk20: f32,       // 20
}
impl_as_bytes!(MotionHeader, 24);

pub fn read_motion(read: &mut CountingReader<impl Read>) -> Result<Motion> {
    debug!(
        "Reading motion header ({}) at {}",
        MotionHeader::SIZE,
        read.offset
    );
    let header: MotionHeader = read.read_struct()?;
    trace!("{:#?}", header);

    assert_that!("motion version", header.version == VERSION, read.prev)?;
    assert_that!("motion loop time", header.loop_time > 0.0, read.prev + 4)?;
    assert_that!("motion field 16", header.unk16 == -1.0, read.prev + 16)?;
    assert_that!("motion field 20", header.unk20 == 1.0, read.prev + 20)?;

    let frame_count = header.frame_count;
    let parts = (0..header.part_count)
        .map(|index| {
            debug!("Reading motion part {} at {}", index, read.offset);
            let part_name = read.read_string()?;
            trace!("Motion part {}/`{}`", index, part_name);

            let flags = read.read_u32()?;
            assert_that!("motion part flags", flags == FLAGS, read.prev)?;

            trace!(
                "Reading motion part {} translations at {}",
                index,
                read.offset
            );

            let mut translations = (0..=frame_count)
                .map(|_| read.read_struct())
                .collect::<std::io::Result<Vec<_>>>()?;

            // the first and last frames always match
            let first = translations.first().ok_or_else(|| {
                assert_with_msg!(
                    "motion part {}/`{}` didn't contain a single frame",
                    index,
                    part_name
                )
            })?;
            let last = translations.last().ok_or_else(|| {
                assert_with_msg!(
                    "motion part {}/`{}` didn't contain a single frame",
                    index,
                    part_name
                )
            })?;
            assert_that!(
                "motion part translation first/last",
                first == last,
                read.offset
            )?;
            translations.pop();

            trace!("Reading motion part {} rotations at {}", index, read.offset);

            let mut rotations = (0..=frame_count)
                .map(|_| read.read_struct())
                .collect::<std::io::Result<Vec<_>>>()?;

            // the first and last frames always match
            let first = rotations.first().ok_or_else(|| {
                assert_with_msg!(
                    "motion part {}/`{}` didn't contain a single frame",
                    index,
                    part_name
                )
            })?;
            let last = rotations.last().ok_or_else(|| {
                assert_with_msg!(
                    "motion part {}/`{}` didn't contain a single frame",
                    index,
                    part_name
                )
            })?;
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

    debug!("Finished reading motions at {}", read.offset);
    read.assert_end()?;

    Ok(Motion {
        loop_time: header.loop_time,
        parts,
        frame_count,
    })
}

pub fn write_motion(write: &mut CountingWriter<impl Write>, motion: &Motion) -> Result<()> {
    debug!(
        "Writing motion header ({}) at {}",
        MotionHeader::SIZE,
        write.offset
    );
    let part_count = assert_len!(u32, motion.parts.len(), "motion parts")?;
    let header = MotionHeader {
        version: VERSION,
        loop_time: motion.loop_time,
        frame_count: motion.frame_count,
        part_count,
        unk16: -1.0,
        unk20: 1.0,
    };
    trace!("{:#?}", header);
    write.write_struct(&header)?;

    for (index, part) in motion.parts.iter().enumerate() {
        debug!(
            "Writing motion part {}/`{}` at {}",
            index, part.name, write.offset
        );
        write.write_string(&part.name)?;
        write.write_u32(FLAGS)?;

        let frame_count = assert_len!(u32, part.frames.len(), "motion part frames")?;
        assert_that!(
            "motion part frame count",
            frame_count == motion.frame_count,
            write.offset
        )?;

        let first = part.frames.first().ok_or_else(|| {
            assert_with_msg!(
                "motion part {}/`{}` didn't contain a single frame",
                index,
                part.name
            )
        })?;

        trace!(
            "Writing motion part {} translations at {}",
            index,
            write.offset
        );

        for frame in &part.frames {
            write.write_struct(&frame.translation)?;
        }
        write.write_struct(&first.translation)?;

        trace!(
            "Writing motion part {} rotations at {}",
            index,
            write.offset
        );

        for frame in &part.frames {
            write.write_struct(&frame.rotation)?;
        }
        write.write_struct(&first.rotation)?;
    }

    debug!("Finished writing motions at {}", write.offset);
    Ok(())
}
