use super::ScriptObject;
use crate::anim::AnimDef;
use crate::io_ext::{CountingReader, WriteHelper};
use crate::size::ReprSize;
use crate::{assert_that, static_assert_size, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[repr(C)]
struct ObjectOpacityFromToC {
    node_index: u32,
    from_state: i16,
    to_state: i16,
    from_value: f32,
    to_value: f32,
    delta: f32,
    runtime: f32,
}
static_assert_size!(ObjectOpacityFromToC, 24);

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectOpacityFromTo {
    pub node: String,
    pub opacity_from: (f32, i16),
    pub opacity_to: (f32, i16),
    pub runtime: f32,
    pub delta: f32,
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
        // TODO: is opacity 0.0 for state -1?
        assert_that!("object opacity from state", object_opacity.from_state in [-1, 0, 1], read.prev + 4)?;
        assert_that!("object opacity to state", object_opacity.to_state in [-1, 0, 1], read.prev + 6)?;
        assert_that!("object opacity from value", 0.0 <= object_opacity.from_value <= 1.0, read.prev + 8)?;
        assert_that!("object opacity to value", 0.0 <= object_opacity.to_value <= 1.0, read.prev + 12)?;
        // TODO: delta is roughly: (to_value - from_value) / runtime
        assert_that!(
            "object opacity from to runtime",
            object_opacity.runtime > 0.0,
            read.prev + 20
        )?;

        Ok(Self {
            node,
            opacity_from: (object_opacity.from_value, object_opacity.from_state),
            opacity_to: (object_opacity.to_value, object_opacity.to_state),
            runtime: object_opacity.runtime,
            delta: object_opacity.delta,
        })
    }

    fn write<W: Write>(&self, write: &mut W, anim_def: &AnimDef) -> Result<()> {
        let node_index = anim_def.node_to_index(&self.node)? as u32;
        write.write_struct(&ObjectOpacityFromToC {
            node_index,
            from_state: self.opacity_from.1,
            to_state: self.opacity_to.1,
            from_value: self.opacity_from.0,
            to_value: self.opacity_to.0,
            delta: self.delta,
            runtime: self.runtime,
        })?;
        Ok(())
    }
}
