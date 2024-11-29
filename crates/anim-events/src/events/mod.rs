mod animation;
mod call_animation;
mod call_object_connector;
mod control_flow;
mod delta;
mod detonate_weapon;
mod fbfx_color_from_to;
mod fog_state;
mod light_animation;
mod light_state;
mod object_active_state;
mod object_add_child;
mod object_connector;
mod object_cycle_texture;
mod object_motion;
mod object_motion_from_to;
mod object_motion_si_script;
mod object_opacity_from_to;
mod object_opacity_state;
mod object_rotate_state;
mod object_scale_state;
mod object_translate_state;
mod parse;
mod puffer_state;
mod sequence;
mod sound;
mod sound_node;
mod types;

use mech3ax_api_types::anim::AnimDef;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::Result;
use object_motion_si_script::object_motion_si_script_size;
pub use parse::{read_events, size_events, write_events};
use std::io::{Read, Write};

pub trait ScriptObject: Sized {
    const INDEX: u8;
    const SIZE: u32;
    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self>;
    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()>;
}
