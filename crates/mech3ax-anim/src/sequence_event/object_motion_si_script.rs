use super::ScriptObject;
use crate::AnimDef;
use ::serde::{Deserialize, Serialize};
use mech3ax_common::assert::AssertionError;
use mech3ax_common::io_ext::{CountingReader, WriteHelper};
use mech3ax_common::serde::base64;
use mech3ax_common::size::ReprSize;
use mech3ax_common::string::bytes_to_c;
use mech3ax_common::types::{Vec3, Vec4};
use mech3ax_common::{assert_that, static_assert_size, Result};
use std::io::{Read, Write};

#[repr(C)]
struct ScriptHeaderC {
    node_index: u32, // 00
    count: u32,      // 04
    zero08: f32,     // 08
    zero12: f32,     // 12
    zero16: u32,     // 16
    zero20: u32,     // 20
}
static_assert_size!(ScriptHeaderC, 24);

#[repr(C)]
struct FrameC {
    flags: u32,
    start_time: f32,
    end_time: f32,
}
static_assert_size!(FrameC, 12);

#[repr(C)]
struct TranslateDataC {
    value: Vec3,
    unk: [u8; 64],
}
static_assert_size!(TranslateDataC, 76);

#[derive(Debug, Serialize, Deserialize)]
pub struct TranslateData {
    pub value: Vec3,
    #[serde(with = "base64")]
    pub unk: Vec<u8>,
}

#[repr(C)]
struct RotateDataC {
    value: Vec4,
    unk: [u8; 60],
}
static_assert_size!(RotateDataC, 76);

#[derive(Debug, Serialize, Deserialize)]
pub struct RotateData {
    pub value: Vec4,
    #[serde(with = "base64")]
    pub unk: Vec<u8>,
}

#[repr(C)]
struct ScaleDataC {
    value: Vec3,
    unk: [u8; 64],
}
static_assert_size!(ScaleDataC, 76);

#[derive(Debug, Serialize, Deserialize)]
pub struct ScaleData {
    pub value: Vec3,
    #[serde(with = "base64")]
    pub unk: Vec<u8>,
}

bitflags::bitflags! {
    pub struct FrameFlags: u32 {
        const TRANSLATE = 1 << 0;
        const ROTATE = 1 << 1;
        const SCALE = 1 << 2;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectMotionSiFrame {
    pub start_time: f32,
    pub end_time: f32,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub translation: Option<TranslateData>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub rotation: Option<RotateData>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub scale: Option<ScaleData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectMotionSiScript {
    pub node: String,
    pub frames: Vec<ObjectMotionSiFrame>,
}

fn read_frame<R: Read>(read: &mut CountingReader<R>) -> Result<ObjectMotionSiFrame> {
    let frame: FrameC = read.read_struct()?;
    let flags = FrameFlags::from_bits(frame.flags).ok_or_else(|| {
        AssertionError(format!(
            "Expected valid object motion si script flags, but was {:08X} (at {})",
            frame.flags,
            read.prev + 0
        ))
    })?;
    assert_that!(
        "object motion si script frame start",
        frame.start_time >= 0.0,
        read.prev + 4
    )?;
    if frame.end_time > 0.0 {
        assert_that!(
            "object motion si script frame end",
            frame.end_time >= frame.start_time,
            read.prev + 8
        )?;
    } else {
        // TODO
    }

    let translation = if flags.contains(FrameFlags::TRANSLATE) {
        let translation: TranslateDataC = read.read_struct()?;
        Some(TranslateData {
            value: translation.value,
            unk: translation.unk.to_vec(),
        })
    } else {
        None
    };

    let rotation = if flags.contains(FrameFlags::ROTATE) {
        let rotation: RotateDataC = read.read_struct()?;
        Some(RotateData {
            value: rotation.value,
            unk: rotation.unk.to_vec(),
        })
    } else {
        None
    };

    let scale = if flags.contains(FrameFlags::SCALE) {
        let scale: ScaleDataC = read.read_struct()?;
        Some(ScaleData {
            value: scale.value,
            unk: scale.unk.to_vec(),
        })
    } else {
        None
    };

    Ok(ObjectMotionSiFrame {
        start_time: frame.start_time,
        end_time: frame.end_time,
        translation,
        rotation,
        scale,
    })
}

impl ScriptObject for ObjectMotionSiScript {
    const INDEX: u8 = 12;
    const SIZE: u32 = u32::MAX;

    fn read<R: Read>(read: &mut CountingReader<R>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        let end_offset = read.offset + size;
        let header: ScriptHeaderC = read.read_struct()?;

        let node = anim_def.node_from_index(header.node_index as usize, read.prev + 0)?;
        assert_that!(
            "object motion si script field 08",
            header.zero08 == 0.0,
            read.prev + 8
        )?;
        assert_that!(
            "object motion si script field 12",
            header.zero12 == 0.0,
            read.prev + 12
        )?;
        assert_that!(
            "object motion si script field 16",
            header.zero16 == 0,
            read.prev + 16
        )?;
        assert_that!(
            "object motion si script field 20",
            header.zero20 == 0,
            read.prev + 20
        )?;

        let frames = (0..header.count)
            .map(|_| read_frame(read))
            .collect::<Result<Vec<_>>>()?;

        assert_that!(
            "object motion si script end",
            read.offset == end_offset,
            read.offset
        )?;

        Ok(ObjectMotionSiScript { node, frames })
    }

    fn write<W: Write>(&self, write: &mut W, anim_def: &AnimDef) -> Result<()> {
        let node_index = anim_def.node_to_index(&self.node)? as u32;
        let count = self.frames.len() as u32;
        write.write_struct(&ScriptHeaderC {
            node_index,
            count,
            zero08: 0.0,
            zero12: 0.0,
            zero16: 0,
            zero20: 0,
        })?;

        for frame in &self.frames {
            let mut flags = FrameFlags::empty();
            if frame.translation.is_some() {
                flags |= FrameFlags::TRANSLATE;
            }
            if frame.rotation.is_some() {
                flags |= FrameFlags::ROTATE;
            }
            if frame.scale.is_some() {
                flags |= FrameFlags::SCALE;
            }

            write.write_struct(&FrameC {
                flags: flags.bits(),
                start_time: frame.start_time,
                end_time: frame.end_time,
            })?;

            if let Some(translation) = &frame.translation {
                let mut unk = [0; 64];
                bytes_to_c(&translation.unk, &mut unk);
                write.write_struct(&TranslateDataC {
                    value: translation.value,
                    unk,
                })?;
            }
            if let Some(rotation) = &frame.rotation {
                let mut unk = [0; 60];
                bytes_to_c(&rotation.unk, &mut unk);
                write.write_struct(&RotateDataC {
                    value: rotation.value,
                    unk,
                })?;
            }
            if let Some(scale) = &frame.scale {
                let mut unk = [0; 64];
                bytes_to_c(&scale.unk, &mut unk);
                write.write_struct(&ScaleDataC {
                    value: scale.value,
                    unk,
                })?;
            }
        }

        Ok(())
    }
}

impl ObjectMotionSiScript {
    pub fn size(&self) -> u32 {
        let mut size = ScriptHeaderC::SIZE + self.frames.len() as u32 * FrameC::SIZE;
        for frame in &self.frames {
            if frame.translation.is_some() {
                size += TranslateDataC::SIZE;
            }
            if frame.rotation.is_some() {
                size += RotateDataC::SIZE;
            }
            if frame.scale.is_some() {
                size += ScaleDataC::SIZE;
            }
        }
        size
    }
}
