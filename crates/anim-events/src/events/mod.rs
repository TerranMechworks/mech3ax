mod delta;
mod e01_sound;
mod e02_sound_node;
mod e04_light_state;
mod e05_light_animation;
mod e06_object_active_state;
mod e07_object_translate_state;
mod e08_object_scale_state;
mod e09_object_rotate_state;
mod e10_object_motion;
mod e11_object_motion_from_to;
mod e12_object_motion_si_script;
mod e13_object_opacity_state;
mod e14_object_opacity_from_to;
mod e15_object_add_child;
mod e17_object_cycle_texture;
mod e18_object_connector;
mod e19_call_object_connector;
mod e22_sequence;
mod e24_call_animation;
mod e26_animation;
mod e28_fog_state;
mod e31_control_flow;
mod e36_fbfx_color_from_to;
mod e41_detonate_weapon;
mod e42_puffer_state;
mod parse;
mod types;

use e12_object_motion_si_script::object_motion_si_script_size;
use mech3ax_api_types::anim::AnimDef;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::Result;
pub use parse::{read_events, size_events, write_events};
use std::io::{Read, Write};

pub trait ScriptObject: Sized {
    const INDEX: u8;
    const SIZE: u32;
    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self>;
    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()>;
}
