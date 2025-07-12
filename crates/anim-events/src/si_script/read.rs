use super::{FrameC, FrameFlags, RotateDataC, ScaleDataC, TranslateDataC};
use mech3ax_api_types::anim::{ObjectMotionSiFrame, RotateData, ScaleData, TranslateData};
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{Result, assert_that};
use std::io::Read;

pub fn read_si_script_frames(
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

fn read_frame(index: u32, read: &mut CountingReader<impl Read>) -> Result<ObjectMotionSiFrame> {
    log::trace!("Reading object motion frame {}", index);
    let frame: FrameC = read.read_struct()?;

    let flags = assert_that!("object motion frame flags", flags frame.flags, read.prev + 0)?;
    let translate = if flags.contains(FrameFlags::TRANSLATE) {
        let data: TranslateDataC = read.read_struct()?;
        // assert_that!(
        //     "object motion frame translate unk",
        //     data.unk == 0.0,
        //     read.prev + 12
        // )?;

        Some(TranslateData {
            base: data.base,
            delta: data.delta,
            garbage: data.unk,
            spline_x: data.spline_x.to_vec(),
            spline_y: data.spline_y.to_vec(),
            spline_z: data.spline_z.to_vec(),
        })
    } else {
        None
    };

    let rotate = if flags.contains(FrameFlags::ROTATE) {
        let data: RotateDataC = read.read_struct()?;
        Some(RotateData {
            base: data.base,
            delta: data.delta,
            spline_x: data.spline_x.to_vec(),
            spline_y: data.spline_y.to_vec(),
            spline_z: data.spline_z.to_vec(),
        })
    } else {
        None
    };

    let scale = if flags.contains(FrameFlags::SCALE) {
        let data: ScaleDataC = read.read_struct()?;
        // assert_that!(
        //     "object motion frame scale unk",
        //     data.unk == 0.0,
        //     read.prev + 12
        // )?;

        Some(ScaleData {
            base: data.base,
            delta: data.delta,
            garbage: data.unk,
            spline_x: data.spline_x.to_vec(),
            spline_y: data.spline_y.to_vec(),
            spline_z: data.spline_z.to_vec(),
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
