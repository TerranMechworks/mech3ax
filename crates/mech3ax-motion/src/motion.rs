use mech3ax_api_types::{static_assert_size, Motion, MotionFrame, MotionPart};
use mech3ax_common::assert::AssertionError;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use std::io::{Read, Write};

const VERSION: u32 = 4;
const FLAGS: u32 = 8 | 4;

#[repr(C)]
struct Header {
    version: u32,
    loop_time: f32,
    frame_count: u32,
    part_count: u32,
    minus_one: f32,
    plus_one: f32,
}
static_assert_size!(Header, 24);

pub fn read_motion(read: &mut CountingReader<impl Read>) -> Result<Motion> {
    let header: Header = read.read_struct()?;
    assert_that!("version", header.version == VERSION, read.prev)?;
    assert_that!("loop time", header.loop_time > 0.0, read.prev + 4)?;
    assert_that!("field 16", header.minus_one == -1.0, read.prev + 16)?;
    assert_that!("field 20", header.plus_one == 1.0, read.prev + 20)?;

    let frame_count = header.frame_count;
    let parts = (0..header.part_count)
        .map(|_| {
            let part_name = read.read_string()?;
            let flags = read.read_u32()?;
            // 8 = translation, 4 = rotation, 2 = scaling (never in motion.zbd)
            assert_that!("flag", flags == FLAGS, read.prev)?;

            let mut translations = (0..=frame_count)
                .map(|_| read.read_struct())
                .collect::<std::io::Result<Vec<_>>>()?;

            // the first and last frames always match
            let first = translations.first().ok_or_else(|| {
                AssertionError(format!(
                    "part `{}` didn't contain a single frame",
                    part_name
                ))
            })?;
            let last = translations.last().ok_or_else(|| {
                AssertionError(format!(
                    "part `{}` didn't contain a single frame",
                    part_name
                ))
            })?;
            assert_that!("part translation first/last", first == last, read.offset)?;
            translations.pop();

            let mut rotations = (0..=frame_count)
                .map(|_| read.read_struct())
                .collect::<std::io::Result<Vec<_>>>()?;

            // the first and last frames always match
            let first = rotations.first().ok_or_else(|| {
                AssertionError(format!(
                    "part `{}` didn't contain a single frame",
                    part_name
                ))
            })?;
            let last = rotations.last().ok_or_else(|| {
                AssertionError(format!(
                    "part `{}` didn't contain a single frame",
                    part_name
                ))
            })?;
            assert_that!("part rotation first/last", first == last, read.offset)?;
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

pub fn write_motion(write: &mut CountingWriter<impl Write>, motion: &Motion) -> Result<()> {
    let header = Header {
        version: VERSION,
        loop_time: motion.loop_time,
        frame_count: motion.frame_count,
        part_count: motion.parts.len() as u32,
        minus_one: -1.0,
        plus_one: 1.0,
    };
    write.write_struct(&header)?;

    for part in &motion.parts {
        write.write_string(&part.name)?;
        write.write_u32(FLAGS)?;

        let first = part.frames.first();

        for frame in &part.frames {
            write.write_struct(&frame.translation)?;
        }
        if let Some(first) = first {
            write.write_struct(&first.translation)?;
        }

        for frame in &part.frames {
            write.write_struct(&frame.rotation)?;
        }
        if let Some(first) = first {
            write.write_struct(&first.rotation)?;
        }
    }
    Ok(())
}
