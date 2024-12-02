use super::EventPm;
use crate::types::{AnimDefLookup as _, Idx32};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::ObjectMotionSiScript;
use mech3ax_api_types::anim::AnimDef;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{impl_as_bytes, AsBytes as _};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ScriptHeaderPmC {
    node_index: Idx32, // 00
    script_index: u32, // 04
    unk08: f32,        // 08
    script_time: f32,  // 12
    script_pos: u32,   // 16
    frame_index: u32,  // 20
}
impl_as_bytes!(ScriptHeaderPmC, 24);

impl EventPm for ObjectMotionSiScript {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(ScriptHeaderPmC::SIZE)
    }

    #[inline]
    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!(
            "object motion si script size",
            size == ScriptHeaderPmC::SIZE,
            read.offset
        )?;
        let header: ScriptHeaderPmC = read.read_struct()?;

        let name = anim_def.node_from_index(header.node_index, read.prev + 0)?;
        assert_that!(
            "object motion si script field 08",
            header.unk08 == 0.0,
            read.prev + 8
        )?;
        assert_that!(
            "object motion si script time",
            header.script_time == 0.0,
            read.prev + 12
        )?;
        assert_that!(
            "object motion si script position",
            header.script_pos == 0,
            read.prev + 16
        )?;
        assert_that!(
            "object motion si script frame index",
            header.frame_index == 0,
            read.prev + 20
        )?;

        Ok(Self {
            name,
            index: header.script_index,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let node_index = anim_def.node_to_index(&self.name)?;

        let header = ScriptHeaderPmC {
            node_index,
            script_index: self.index,
            unk08: 0.0,
            script_time: 0.0,
            script_pos: 0,
            frame_index: 0,
        };
        write.write_struct(&header)?;

        Ok(())
    }
}
