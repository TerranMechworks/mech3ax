mod delta;
mod e01_sound;
mod e02_sound_node;
mod e03_effect;
mod e04_light_state;
mod e05_light_animation;
mod e06_object_active_state;
mod e07_object_translate_state;
mod e08_object_scale_state;
mod e09_object_rotate_state;
mod e10_object_motion;
mod e11_object_motion_from_to;
pub(crate) mod e12_object_motion_si_script;
mod e13_object_opacity_state;
mod e14_object_opacity_from_to;
mod e15_object_add_child;
mod e16_object_delete_child;
mod e17_object_cycle_texture;
mod e18_object_connector;
mod e19_call_object_connector;
mod e20_camera_state;
mod e21_camera_from_to;
mod e22_sequence;
mod e24_call_animation;
mod e25_stop_animation;
mod e26_animation;
mod e28_fog_state;
mod e30_loop;
mod e31_control_flow;
mod e35_callback;
mod e36_fbfx_color_from_to;
mod e37_fbfx_csinwave_from_to;
mod e39_anim_verbose;
mod e41_detonate_weapon;
mod e42_puffer_state;

pub(crate) use e12_object_motion_si_script as object_motion_si_script;
use mech3ax_api_types::anim::AnimDef;
use mech3ax_common::Result;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use std::io::{Read, Write};

pub(crate) trait EventMw: Sized {
    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self>;
    fn size(&self) -> Option<u32>;
    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()>;
}

pub(crate) trait EventPm: Sized {
    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self>;
    fn size(&self) -> Option<u32>;
    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()>;
}

pub(crate) trait EventRc: Sized {
    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self>;
    fn size(&self) -> Option<u32>;
    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()>;
}

pub(crate) trait EventAll: Sized {
    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self>;
    fn size(&self) -> Option<u32>;
    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()>;
}

impl<T: EventAll> EventMw for T {
    #[inline]
    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        <T as EventAll>::read(read, anim_def, size)
    }

    #[inline]
    fn size(&self) -> Option<u32> {
        <T as EventAll>::size(self)
    }

    #[inline]
    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        <T as EventAll>::write(self, write, anim_def)
    }
}

impl<T: EventAll> EventPm for T {
    #[inline]
    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        <T as EventAll>::read(read, anim_def, size)
    }

    #[inline]
    fn size(&self) -> Option<u32> {
        <T as EventAll>::size(self)
    }

    #[inline]
    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        <T as EventAll>::write(self, write, anim_def)
    }
}

impl<T: EventAll> EventRc for T {
    #[inline]
    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        <T as EventAll>::read(read, anim_def, size)
    }

    #[inline]
    fn size(&self) -> Option<u32> {
        <T as EventAll>::size(self)
    }

    #[inline]
    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        <T as EventAll>::write(self, write, anim_def)
    }
}
