use crate::anim::AnimDef;
use crate::io_ext::CountingReader;
use crate::Result;
use std::io::{Read, Write};

mod animation;
mod call_animation;
mod call_object_connector;
mod control_flow;
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
mod utils;

pub use parse::{read_events, size_events, write_events, Event, EventData, StartOffset};

pub trait ScriptObject: Sized {
    const INDEX: u8;
    const SIZE: u32;
    fn read<R: Read>(read: &mut CountingReader<R>, anim_def: &AnimDef, size: u32) -> Result<Self>;
    fn write<W: Write>(&self, write: &mut W, anim_def: &AnimDef) -> Result<()>;
}
