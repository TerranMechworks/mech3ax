use super::delta::delta;
use super::EventAll;
use crate::types::{AnimDefLookup as _, Idx32};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::{ObjectOpacity, ObjectOpacityFromTo};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, assert_with_msg, Result};
use mech3ax_types::{impl_as_bytes, AsBytes as _};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ObjectOpacityFromToC {
    node_index: Idx32,  // 00
    state_from: i16,    // 04
    state_to: i16,      // 06
    opacity_from: f32,  // 08
    opacity_to: f32,    // 12
    opacity_delta: f32, // 16
    run_time: f32,      // 20
}
impl_as_bytes!(ObjectOpacityFromToC, 24);

fn state_from_i16(name: &str, state: i16, offset: usize) -> Result<Option<bool>> {
    match state {
        -1 => Ok(None),
        0 => Ok(Some(false)),
        1 => Ok(Some(true)),
        _ => Err(assert_with_msg!(
            "Expected `{}` to be -1, 0, or 1, but was {} (at {})",
            name,
            state,
            offset,
        )),
    }
}

fn state_to_i16(state: Option<bool>) -> i16 {
    match state {
        None => -1,
        Some(false) => 0,
        Some(true) => 1,
    }
}

impl EventAll for ObjectOpacityFromTo {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(ObjectOpacityFromToC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!(
            "object opacity from to size",
            size == ObjectOpacityFromToC::SIZE,
            read.offset
        )?;
        let from_to: ObjectOpacityFromToC = read.read_struct()?;

        assert_that!(
            "object opacity from to run time",
            from_to.run_time > 0.0,
            read.prev + 20
        )?;

        let name = anim_def.node_from_index(from_to.node_index, read.prev + 0)?;
        let from_state = state_from_i16(
            "object opacity from state",
            from_to.state_from,
            read.prev + 4,
        )?;
        let to_state = state_from_i16("object opacity to state", from_to.state_to, read.prev + 4)?;
        assert_that!("object opacity from opacity", 0.0 <= from_to.opacity_from <= 1.0, read.prev + 8)?;
        assert_that!("object opacity to opacity", 0.0 <= from_to.opacity_to <= 1.0, read.prev + 12)?;

        // TODO
        let opacity_delta = delta(from_to.opacity_from, from_to.opacity_to, from_to.run_time);
        let opacity_delta = if from_to.opacity_delta == opacity_delta {
            log::trace!("object opacity from to opacity delta: OK");
            None
        } else {
            log::debug!("object opacity from to opacity delta: FAIL");
            log::error!(
                "DELTA VAL FAIL: `{}`, `{}`",
                anim_def.anim_name,
                anim_def.anim_root_name
            );
            Some(from_to.opacity_delta)
        };
        // #[allow(clippy::float_cmp)]
        // let fudge = if object_opacity.delta_value != delta_value {
        //     // some values in c3/anim.zbd of v1.0-us-pre and v1.1-us-pre have slightly
        //     // incorrect values here - see `fbfx_color_from_to.rs`
        //     delta_value = delta(
        //         object_opacity.to_value,
        //         object_opacity.from_value,
        //         dec_f32(object_opacity.run_time),
        //     );
        //     true
        // } else {
        //     false
        // };
        // assert_that!(
        //     "object opacity delta value",
        //     from_to.delta_value == delta_value,
        //     read.prev + 16
        // )?;

        Ok(Self {
            name,
            opacity_from: ObjectOpacity {
                opacity: from_to.opacity_from,
                state: from_state,
            },
            opacity_to: ObjectOpacity {
                opacity: from_to.opacity_to,
                state: to_state,
            },
            run_time: from_to.run_time,
            opacity_delta,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let node_index = anim_def.node_to_index(&self.name)?;
        let run_time = self.run_time;

        let state_from = state_to_i16(self.opacity_from.state);
        let state_to = state_to_i16(self.opacity_to.state);
        let opacity_from = self.opacity_from.opacity;
        let opacity_to = self.opacity_to.opacity;

        // TODO
        // let run_time = if self.fudge {
        //     dec_f32(self.run_time)
        // } else {
        //     self.run_time
        // };
        let opacity_delta = self
            .opacity_delta
            .unwrap_or_else(|| delta(opacity_from, opacity_to, run_time));

        let from_to = ObjectOpacityFromToC {
            node_index,
            state_from,
            state_to,
            opacity_from,
            opacity_to,
            opacity_delta,
            run_time: self.run_time,
        };
        write.write_struct(&from_to)?;
        Ok(())
    }
}
