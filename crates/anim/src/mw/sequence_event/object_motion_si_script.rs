use super::ScriptObject;
use crate::types::AnimDefLookup as _;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::{
    ObjectMotionSiFrame, ObjectMotionSiScript, RotateData, ScaleData, TranslateData,
};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::{Quaternion, Vec3};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, assert_with_msg, Result};
use mech3ax_types::{impl_as_bytes, u32_to_usize, AsBytes as _, Bytes};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ScriptHeaderC {
    node_index: u32, // 00
    count: u32,      // 04
    zero08: f32,     // 08
    zero12: f32,     // 12
    zero16: u32,     // 16
    zero20: u32,     // 20
}
impl_as_bytes!(ScriptHeaderC, 24);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct FrameC {
    flags: u32,
    start_time: f32,
    end_time: f32,
}
impl_as_bytes!(FrameC, 12);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct TranslateDataC {
    value: Vec3,
    unk: Bytes<64>,
}
impl_as_bytes!(TranslateDataC, 76);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct RotateDataC {
    value: Quaternion,
    unk: Bytes<60>,
}
impl_as_bytes!(RotateDataC, 76);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ScaleDataC {
    value: Vec3,
    unk: Bytes<64>,
}
impl_as_bytes!(ScaleDataC, 76);

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FrameFlags: u32 {
        const TRANSLATE = 1 << 0;
        const ROTATE = 1 << 1;
        const SCALE = 1 << 2;
    }
}

fn read_frame(read: &mut CountingReader<impl Read>) -> Result<ObjectMotionSiFrame> {
    let frame: FrameC = read.read_struct()?;
    let flags = FrameFlags::from_bits(frame.flags).ok_or_else(|| {
        assert_with_msg!(
            "Expected valid object motion si script flags, but was {:08X} (at {})",
            frame.flags,
            read.prev + 0
        )
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

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        let end_offset = read.offset + u32_to_usize(size);
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

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
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
                let unk = Bytes::from_slice(&translation.unk);
                write.write_struct(&TranslateDataC {
                    value: translation.value,
                    unk,
                })?;
            }
            if let Some(rotation) = &frame.rotation {
                let unk = Bytes::from_slice(&rotation.unk);
                write.write_struct(&RotateDataC {
                    value: rotation.value,
                    unk,
                })?;
            }
            if let Some(scale) = &frame.scale {
                let unk = Bytes::from_slice(&scale.unk);
                write.write_struct(&ScaleDataC {
                    value: scale.value,
                    unk,
                })?;
            }
        }

        Ok(())
    }
}

pub fn object_motion_si_script_size(script: &ObjectMotionSiScript) -> u32 {
    let mut size = ScriptHeaderC::SIZE + script.frames.len() as u32 * FrameC::SIZE;
    for frame in &script.frames {
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
