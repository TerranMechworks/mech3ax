use crate::io_ext::{FromUsize, ReadHelper, WriteHelper};
use crate::size::ReprSize;
use crate::{assert_that, static_assert_size, Result};
use ::serde::{Deserialize, Serialize};
use std::io::{Read, Write};

const VERSION: u32 = 4;

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

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct Vector(f32, f32, f32);
static_assert_size!(Vector, 12);

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct Quarternion(f32, f32, f32, f32);
static_assert_size!(Quarternion, 16);

#[derive(Debug, Serialize, Deserialize)]
pub struct Frame {
    translation: Vector,
    rotation: Quarternion,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Motion {
    loop_time: f32,
    // need to preserve order
    parts: Vec<(String, Vec<Frame>)>,
    frame_count: u32,
}

impl FromUsize for u32 {
    fn from_usize(value: usize) -> Self {
        value as Self
    }
}

pub fn read_motion<R>(read: &mut R) -> Result<Motion>
where
    R: Read,
{
    let header: Header = read.read_struct()?;
    assert_that!("version", header.version == VERSION, 0)?;
    assert_that!("loop time", header.loop_time > 0.0, 4)?;
    assert_that!("field 16", header.minus_one == -1.0, 16)?;
    assert_that!("field 20", header.plus_one == 1.0, 20)?;

    let frame_count = header.frame_count + 1;
    let mut offset = Header::SIZE;
    let parts = (0..header.part_count)
        .into_iter()
        .map(|_| {
            let part_name = read.read_string(&mut offset)?;
            let flag = read.read_u32()?;
            // 8 = translation, 4 = rotation, 2 = scaling (never in motion.zbd)
            assert_that!("flag", flag == 12, offset)?;
            offset += 4;

            let translations = (0..frame_count)
                .into_iter()
                .map(|_| {
                    let value = read.read_struct::<Vector>()?;
                    Ok(value)
                })
                .collect::<Result<Vec<_>>>()?;
            offset += Vector::SIZE * frame_count;

            let rotations = (0..frame_count)
                .into_iter()
                .map(|_| {
                    let value = read.read_struct::<Quarternion>()?;
                    Ok(value)
                })
                .collect::<Result<Vec<_>>>()?;
            offset += Quarternion::SIZE * frame_count;

            let frames = translations
                .into_iter()
                .zip(rotations.into_iter())
                .map(|(translation, rotation)| Frame {
                    translation,
                    rotation,
                })
                .collect();
            Ok((part_name, frames))
        })
        .collect::<Result<Vec<_>>>()?;

    read.assert_end()?;
    Ok(Motion {
        loop_time: header.loop_time,
        parts,
        frame_count,
    })
}

pub fn write_motion<W>(write: &mut W, motion: Motion) -> Result<()>
where
    W: Write,
{
    let header = Header {
        version: VERSION,
        loop_time: motion.loop_time,
        frame_count: motion.frame_count - 1,
        part_count: motion.parts.len() as u32,
        minus_one: -1.0,
        plus_one: 1.0,
    };
    write.write_struct(&header)?;

    for (part_name, frames) in motion.parts {
        write.write_string(part_name)?;
        write.write_u32(12)?; // flag

        for frame in &frames {
            write.write_struct(&frame.translation)?;
        }

        for frame in &frames {
            write.write_struct(&frame.rotation)?;
        }
    }
    Ok(())
}
