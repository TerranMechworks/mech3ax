use super::{MotionFlags, MotionHeaderC, VERSION};
use log::trace;
use mech3ax_api_types::motion::Motion;
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{assert_len, assert_that, Result};
use std::io::Write;

pub fn write_motion(write: &mut CountingWriter<impl Write>, motion: &Motion) -> Result<()> {
    let part_count = assert_len!(u32, motion.parts.len(), "motion parts")?;
    assert_that!("motion frame count", motion.frame_count > 0, 0)?;

    let header = MotionHeaderC {
        version: VERSION,
        loop_time: motion.loop_time,
        frame_count: motion.frame_count,
        part_count,
        unk16: -1.0,
        unk20: 1.0,
    };
    write.write_struct(&header)?;

    for (index, part) in motion.parts.iter().enumerate() {
        trace!("Writing motion part {}", index);
        write.write_string(&part.name)?;
        write.write_u32(MotionFlags::DEFAULT.bits())?;

        let frame_count = assert_len!(u32, part.frames.len(), "motion part frames")?;
        assert_that!(
            "motion part frame count",
            frame_count == motion.frame_count,
            write.offset
        )?;

        let first = part.frames.first().unwrap();

        for frame in &part.frames {
            write.write_struct_no_log(&frame.translation)?;
        }
        write.write_struct_no_log(&first.translation)?;

        for frame in &part.frames {
            write.write_struct_no_log(&frame.rotation)?;
        }
        write.write_struct_no_log(&first.rotation)?;
    }

    Ok(())
}
