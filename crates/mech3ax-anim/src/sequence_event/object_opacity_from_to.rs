use super::delta::{dec_f32, delta};
use super::ScriptObject;
use crate::AnimDef;
use ::serde::{Deserialize, Serialize};
use mech3ax_api_types::serde::bool_false;
use mech3ax_api_types::{static_assert_size, ReprSize as _};
use mech3ax_common::io_ext::{CountingReader, WriteHelper};
use mech3ax_common::{assert_that, Result};
use std::io::{Read, Write};

#[repr(C)]
struct ObjectOpacityFromToC {
    node_index: u32,
    from_state: i16,
    to_state: i16,
    from_value: f32,
    to_value: f32,
    delta_value: f32,
    runtime: f32,
}
static_assert_size!(ObjectOpacityFromToC, 24);

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectOpacityFromTo {
    pub node: String,
    pub opacity_from: (f32, i16),
    pub opacity_to: (f32, i16),
    pub runtime: f32,
    // this value can be safely ignored, but is required for binary accuracy
    #[serde(skip_serializing_if = "bool_false", default)]
    pub fudge: bool,
}

impl ScriptObject for ObjectOpacityFromTo {
    const INDEX: u8 = 14;
    const SIZE: u32 = ObjectOpacityFromToC::SIZE;

    fn read<R: Read>(read: &mut CountingReader<R>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!(
            "object opacity from to size",
            size == Self::SIZE,
            read.offset
        )?;
        let object_opacity: ObjectOpacityFromToC = read.read_struct()?;
        let node = anim_def.node_from_index(object_opacity.node_index as usize, read.prev + 0)?;
        // the opacity value is independent from the state; e.g. for -1, opacity is not necessarily 0.0
        assert_that!("object opacity from state", object_opacity.from_state in [-1, 0, 1], read.prev + 4)?;
        assert_that!("object opacity to state", object_opacity.to_state in [-1, 0, 1], read.prev + 6)?;
        assert_that!("object opacity from value", 0.0 <= object_opacity.from_value <= 1.0, read.prev + 8)?;
        assert_that!("object opacity to value", 0.0 <= object_opacity.to_value <= 1.0, read.prev + 12)?;
        assert_that!(
            "object opacity from to runtime",
            object_opacity.runtime > 0.0,
            read.prev + 20
        )?;
        let mut delta_value = delta(
            object_opacity.to_value,
            object_opacity.from_value,
            object_opacity.runtime,
        );

        #[allow(clippy::float_cmp)]
        let fudge = if object_opacity.delta_value != delta_value {
            // some values in c3/anim.zbd of v1.0-us-pre and v1.1-us-pre have slightly
            // incorrect values here - see `fbfx_color_from_to.rs`
            delta_value = delta(
                object_opacity.to_value,
                object_opacity.from_value,
                dec_f32(object_opacity.runtime),
            );
            true
        } else {
            false
        };
        assert_that!(
            "object opacity delta value",
            object_opacity.delta_value == delta_value,
            read.prev + 16
        )?;

        Ok(Self {
            node,
            opacity_from: (object_opacity.from_value, object_opacity.from_state),
            opacity_to: (object_opacity.to_value, object_opacity.to_state),
            runtime: object_opacity.runtime,
            fudge,
        })
    }

    fn write<W: Write>(&self, write: &mut W, anim_def: &AnimDef) -> Result<()> {
        let node_index = anim_def.node_to_index(&self.node)? as u32;
        let runtime = if self.fudge {
            dec_f32(self.runtime)
        } else {
            self.runtime
        };
        let delta_value = delta(self.opacity_to.0, self.opacity_from.0, runtime);
        write.write_struct(&ObjectOpacityFromToC {
            node_index,
            from_state: self.opacity_from.1,
            to_state: self.opacity_to.1,
            from_value: self.opacity_from.0,
            to_value: self.opacity_to.0,
            delta_value,
            runtime: self.runtime,
        })?;
        Ok(())
    }
}
