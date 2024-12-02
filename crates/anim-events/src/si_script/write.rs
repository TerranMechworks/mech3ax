use super::{FrameC, FrameFlags, RotateDataC, ScaleDataC, TranslateDataC};
use mech3ax_api_types::anim::ObjectMotionSiFrame;
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::Result;
use mech3ax_types::{AsBytes as _, Bytes};
use std::io::Write;

pub fn size_si_script_frames(frames: &[ObjectMotionSiFrame]) -> Option<u32> {
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

pub fn write_si_script_frames(
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
                spline_x: Bytes::from_slice(&data.spline_x),
                spline_y: Bytes::from_slice(&data.spline_y),
                spline_z: Bytes::from_slice(&data.spline_z),
            };
            write.write_struct(&translate)?;
        }

        if let Some(data) = &frame.rotate {
            let rotate = RotateDataC {
                base: data.base,
                delta: data.delta,
                spline_x: Bytes::from_slice(&data.spline_x),
                spline_y: Bytes::from_slice(&data.spline_y),
                spline_z: Bytes::from_slice(&data.spline_z),
            };
            write.write_struct(&rotate)?;
        }

        if let Some(data) = &frame.scale {
            let scale = ScaleDataC {
                base: data.base,
                unk: data.garbage,
                delta: data.delta,
                spline_x: Bytes::from_slice(&data.spline_x),
                spline_y: Bytes::from_slice(&data.spline_y),
                spline_z: Bytes::from_slice(&data.spline_z),
            };
            write.write_struct(&scale)?;
        }
    }

    Ok(())
}
