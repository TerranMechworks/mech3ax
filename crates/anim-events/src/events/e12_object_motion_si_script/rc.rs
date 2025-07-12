use crate::types::{AnimDefLookup as _, Idx32};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::ObjectMotionSiScript;
use mech3ax_api_types::anim::{
    AnimDef, ObjectMotionSiFrame, RotateData, ScaleData, SiScript, TranslateData,
};
use mech3ax_api_types::{Quaternion, Vec3};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{Result, assert_that, assert_with_msg};
use mech3ax_types::{AsBytes as _, Maybe, bitflags, impl_as_bytes, u32_to_usize};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ScriptHeaderRcC {
    node_index: Idx32, // 00
    frame_count: u32,  // 04
    script_time: f32,  // 08
    script_pos: u32,   // 12
    frame_index: u32,  // 16
}
impl_as_bytes!(ScriptHeaderRcC, 20);

bitflags! {
    struct FrameFlags: u32 {
        const TRANSLATE = 1 << 0;
        const ROTATE = 1 << 1;
        const SCALE = 1 << 2;
    }
}

type Flags = Maybe<u32, FrameFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct FrameC {
    flags: Flags,    // 00
    start_time: f32, // 04
    end_time: f32,   // 08
}
impl_as_bytes!(FrameC, 12);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct TranslateDataC {
    base: Vec3, // 00
    unk: u32,   // 12
    delta: Vec3, // 16,

                // v0: f32,
                // v1: f32,
                // v2: f32,
                // v3: f32,
                // v4: f32,
                // v5: f32,
                // v6: f32,
}
impl_as_bytes!(TranslateDataC, 28);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct RotateDataC {
    base: Quaternion, // 00
    delta: Vec3,      // 16

                      // v0: f32,
                      // v1: f32,
                      // v2: f32,
                      // v3: f32,
                      // v4: f32,
                      // v5: f32,
                      // v6: f32,
}
impl_as_bytes!(RotateDataC, 28);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ScaleDataC {
    base: Vec3, // 00
    unk: u32,   // 12
    delta: Vec3, // 16,

                // v0: f32,
                // v1: f32,
                // v2: f32,
                // v3: f32,
                // v4: f32,
                // v5: f32,
                // v6: f32,
}
impl_as_bytes!(ScaleDataC, 28);

fn read_frame(index: u32, read: &mut CountingReader<impl Read>) -> Result<ObjectMotionSiFrame> {
    log::trace!("Reading object motion frame {}", index);
    let frame: FrameC = read.read_struct()?;

    let flags = assert_that!("object motion frame flags", flags frame.flags, read.prev + 0)?;
    let translate = if flags.contains(FrameFlags::TRANSLATE) {
        let data: TranslateDataC = read.read_struct()?;
        if data.unk == 0 {
            log::trace!("object motion si script translate: OK");
        } else {
            log::debug!("object motion si script translate: FAIL");
        }
        // assert_that!(
        //     "object motion frame translate unk",
        //     data.unk == 0.0,
        //     read.prev + 12
        // )?;
        Some(TranslateData {
            base: data.base,
            delta: data.delta,
            garbage: data.unk,
            spline_x: Vec::new(),
            spline_y: Vec::new(),
            spline_z: Vec::new(),
        })
    } else {
        None
    };

    let rotate = if flags.contains(FrameFlags::ROTATE) {
        let data: RotateDataC = read.read_struct()?;
        Some(RotateData {
            base: data.base,
            delta: data.delta,
            spline_x: Vec::new(),
            spline_y: Vec::new(),
            spline_z: Vec::new(),
        })
    } else {
        None
    };

    let scale = if flags.contains(FrameFlags::SCALE) {
        let data: ScaleDataC = read.read_struct()?;
        if data.unk == 0 {
            log::trace!("object motion si script scale: OK");
        } else {
            log::debug!("object motion si script scale: FAIL");
        }
        // assert_that!(
        //     "object motion frame scale unk",
        //     data.unk == 0.0,
        //     read.prev + 12
        // )?;
        Some(ScaleData {
            base: data.base,
            delta: data.delta,
            garbage: data.unk,
            spline_x: Vec::new(),
            spline_y: Vec::new(),
            spline_z: Vec::new(),
        })
    } else {
        None
    };

    Ok(ObjectMotionSiFrame {
        start_time: frame.start_time,
        end_time: frame.end_time,
        translate,
        rotate,
        scale,
    })
}

fn read_si_script_frames(
    read: &mut CountingReader<impl Read>,
    size: usize,
    count: u32,
) -> Result<Vec<ObjectMotionSiFrame>> {
    let end_offset = read.offset + size;

    let frames = (0..count)
        .map(|index| read_frame(index, read))
        .collect::<Result<Vec<_>>>()?;

    assert_that!(
        "object motion si script end",
        read.offset == end_offset,
        read.offset
    )?;

    Ok(frames)
}

fn size_si_script_frames(frames: &[ObjectMotionSiFrame]) -> Option<u32> {
    let count: u32 = frames.len().try_into().ok()?;
    let mut size = count.checked_mul(FrameC::SIZE)?;
    for frame in frames {
        if frame.translate.is_some() {
            size = size.checked_add(TranslateDataC::SIZE)?;
        }
        if frame.rotate.is_some() {
            size = size.checked_add(RotateDataC::SIZE)?;
        }
        if frame.scale.is_some() {
            size = size.checked_add(ScaleDataC::SIZE)?;
        }
    }
    Some(size)
}

pub(crate) fn size_rc(data: &ObjectMotionSiScript, scripts: &[SiScript]) -> Option<u32> {
    let index = u32_to_usize(data.index);
    let script = scripts.get(index)?;
    size_si_script_frames(&script.frames)?.checked_add(ScriptHeaderRcC::SIZE)
}

pub(crate) fn read_rc(
    read: &mut CountingReader<impl Read>,
    anim_def: &AnimDef,
    size: u32,
    scripts: &mut Vec<SiScript>,
) -> Result<ObjectMotionSiScript> {
    let size = size.checked_sub(ScriptHeaderRcC::SIZE).ok_or_else(|| {
        assert_with_msg!(
            "Expected `object motion si script size` > {}, but was {} (at {})",
            ScriptHeaderRcC::SIZE,
            size,
            read.offset
        )
    })?;
    let header: ScriptHeaderRcC = read.read_struct()?;

    let name = anim_def.node_from_index(header.node_index, read.prev + 0)?;
    assert_that!(
        "object motion si script time",
        header.script_time == 0.0,
        read.prev + 8
    )?;
    assert_that!(
        "object motion si script position",
        header.script_pos == 0,
        read.prev + 12
    )?;
    assert_that!(
        "object motion si script frame index",
        header.frame_index == 0,
        read.prev + 16
    )?;

    let size = u32_to_usize(size);

    let index = scripts
        .len()
        .try_into()
        .map_err(|_e| assert_with_msg!("Object motion si script overflow (at {})", read.prev))?;

    let frames = read_si_script_frames(read, size, header.frame_count)?;
    let script = SiScript {
        script_name: "undefined".to_string(),
        object_name: "undefined".to_string(),
        frames,
        spline_interp: false,
        script_name_ptr: u32::MAX,
        object_name_ptr: u32::MAX,
        script_data_ptr: u32::MAX,
    };
    scripts.push(script);

    Ok(ObjectMotionSiScript { name, index })
}

fn write_si_script_frames(
    write: &mut CountingWriter<impl Write>,
    frames: &[ObjectMotionSiFrame],
) -> Result<()> {
    for (index, frame) in frames.iter().enumerate() {
        log::trace!("Writing object motion frame {}", index);

        let mut flags = FrameFlags::empty();
        if frame.translate.is_some() {
            flags |= FrameFlags::TRANSLATE;
        }
        if frame.rotate.is_some() {
            flags |= FrameFlags::ROTATE;
        }
        if frame.scale.is_some() {
            flags |= FrameFlags::SCALE;
        }

        let frame_c = FrameC {
            flags: flags.maybe(),
            start_time: frame.start_time,
            end_time: frame.end_time,
        };
        write.write_struct(&frame_c)?;

        if let Some(data) = &frame.translate {
            let translate = TranslateDataC {
                base: data.base,
                unk: data.garbage,
                delta: data.delta,
            };
            write.write_struct(&translate)?;
        }

        if let Some(data) = &frame.rotate {
            let rotate = RotateDataC {
                base: data.base,
                delta: data.delta,
            };
            write.write_struct(&rotate)?;
        }

        if let Some(data) = &frame.scale {
            let scale = ScaleDataC {
                base: data.base,
                unk: data.garbage,
                delta: data.delta,
            };
            write.write_struct(&scale)?;
        }
    }

    Ok(())
}

pub(crate) fn write_rc(
    write: &mut CountingWriter<impl Write>,
    anim_def: &AnimDef,
    data: &ObjectMotionSiScript,
    scripts: &[SiScript],
) -> Result<()> {
    let index = u32_to_usize(data.index);
    let script = scripts
        .get(index)
        .ok_or_else(|| assert_with_msg!("Invalid object motion si script index: {}", data.index))?;

    let node_index = anim_def.node_to_index(&data.name)?;
    let count = script.frames.len() as u32;

    let header = ScriptHeaderRcC {
        node_index,
        frame_count: count,
        script_time: 0.0,
        script_pos: 0,
        frame_index: 0,
    };
    write.write_struct(&header)?;

    write_si_script_frames(write, &script.frames)?;

    Ok(())
}
