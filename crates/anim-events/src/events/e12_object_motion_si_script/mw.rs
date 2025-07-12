use crate::si_script::{read_si_script_frames, size_si_script_frames, write_si_script_frames};
use crate::types::{AnimDefLookup as _, Idx32};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::ObjectMotionSiScript;
use mech3ax_api_types::anim::{AnimDef, SiScript};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{Result, assert_that, assert_with_msg};
use mech3ax_types::{AsBytes as _, impl_as_bytes, u32_to_usize};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ScriptHeaderMwC {
    node_index: Idx32, // 00
    frame_count: u32,  // 04
    unk08: f32,        // 08
    script_time: f32,  // 12
    script_pos: u32,   // 16
    frame_index: u32,  // 20
}
impl_as_bytes!(ScriptHeaderMwC, 24);

pub(crate) fn size_mw(data: &ObjectMotionSiScript, scripts: &[SiScript]) -> Option<u32> {
    let index = u32_to_usize(data.index);
    let script = scripts.get(index)?;
    size_si_script_frames(&script.frames)?.checked_add(ScriptHeaderMwC::SIZE)
}

pub(crate) fn read_mw(
    read: &mut CountingReader<impl Read>,
    anim_def: &AnimDef,
    size: u32,
    scripts: &mut Vec<SiScript>,
) -> Result<ObjectMotionSiScript> {
    let size = size.checked_sub(ScriptHeaderMwC::SIZE).ok_or_else(|| {
        assert_with_msg!(
            "Expected `object motion si script size` > {}, but was {} (at {})",
            ScriptHeaderMwC::SIZE,
            size,
            read.offset
        )
    })?;

    let header: ScriptHeaderMwC = read.read_struct()?;

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

pub(crate) fn write_mw(
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

    let header = ScriptHeaderMwC {
        node_index,
        frame_count: count,
        unk08: 0.0,
        script_time: 0.0,
        script_pos: 0,
        frame_index: 0,
    };
    write.write_struct(&header)?;

    write_si_script_frames(write, &script.frames)?;

    Ok(())
}
