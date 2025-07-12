use super::EventAll;
use super::delta::{FloatFromToC, delta};
use super::e20_camera_state::CameraStateFlags;
use crate::types::{AnimDefLookup as _, Idx32};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::anim::events::{CameraFromTo, FloatFromTo};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{Result, assert_that};
use mech3ax_types::{AsBytes as _, Maybe, impl_as_bytes};
use std::io::{Read, Write};

type Flags = Maybe<u32, CameraStateFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct CameraFromToC {
    flags: Flags,                 // 00
    node_index: Idx32,            // 04
    clip_near: FloatFromToC,      // 08
    clip_far: FloatFromToC,       // 20
    lod_multiplier: FloatFromToC, // 32
    fov_h: FloatFromToC,          // 44
    fov_v: FloatFromToC,          // 56
    zoom_h: FloatFromToC,         // 68
    zoom_v: FloatFromToC,         // 80
    run_time: f32,                // 92
}
impl_as_bytes!(CameraFromToC, 96);

fn assert_flag_and_value(
    base_name: &str,
    flags: CameraStateFlags,
    flag: CameraStateFlags,
    value: FloatFromToC,
    run_time: f32,
    offset: usize,
) -> Result<Option<FloatFromTo>> {
    let delta = delta(value.from, value.to, run_time);

    let name = format!("{} delta", base_name);
    assert_that!(&name, value.delta == delta, offset + 8)?;

    if flags.contains(flag) {
        Ok(Some(FloatFromTo {
            from: value.from,
            to: value.to,
        }))
    } else {
        let name = format!("{} from", base_name);
        assert_that!(&name, value.from == 0.0, offset + 0)?;
        let name = format!("{} to", base_name);
        assert_that!(&name, value.to == 0.0, offset + 4)?;
        Ok(None)
    }
}

fn make_value(value: &Option<FloatFromTo>, run_time: f32) -> FloatFromToC {
    match value {
        Some(value) => {
            let delta = delta(value.from, value.to, run_time);
            FloatFromToC {
                from: value.from,
                to: value.to,
                delta,
            }
        }
        None => FloatFromToC {
            from: 0.0,
            to: 0.0,
            delta: 0.0,
        },
    }
}

impl EventAll for CameraFromTo {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(CameraFromToC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!(
            "camera from to size",
            size == CameraFromToC::SIZE,
            read.offset
        )?;
        let state: CameraFromToC = read.read_struct()?;

        assert_that!(
            "camera from to run time",
            state.run_time > 0.0,
            read.prev + 92
        )?;

        let flags = assert_that!("camera from to flags", flags state.flags, read.prev + 0)?;
        let name = anim_def.node_from_index(state.node_index, read.prev + 4)?;

        let clip_near = assert_flag_and_value(
            "camera from to clip near",
            flags,
            CameraStateFlags::CLIP_NEAR,
            state.clip_near,
            state.run_time,
            read.prev + 8,
        )?;
        let clip_far = assert_flag_and_value(
            "camera from to clip far",
            flags,
            CameraStateFlags::CLIP_FAR,
            state.clip_far,
            state.run_time,
            read.prev + 20,
        )?;
        let lod_multiplier = assert_flag_and_value(
            "camera from to lod multiplier",
            flags,
            CameraStateFlags::LOD_MULTIPLIER,
            state.lod_multiplier,
            state.run_time,
            read.prev + 32,
        )?;
        let fov_h = assert_flag_and_value(
            "camera from to fov h",
            flags,
            CameraStateFlags::FOV_H,
            state.fov_h,
            state.run_time,
            read.prev + 44,
        )?;
        let fov_v = assert_flag_and_value(
            "camera from to fov v",
            flags,
            CameraStateFlags::FOV_V,
            state.fov_v,
            state.run_time,
            read.prev + 56,
        )?;
        let zoom_h = assert_flag_and_value(
            "camera from to zoom h",
            flags,
            CameraStateFlags::ZOOM_H,
            state.zoom_h,
            state.run_time,
            read.prev + 68,
        )?;
        let zoom_v = assert_flag_and_value(
            "camera from to fov zoom v",
            flags,
            CameraStateFlags::ZOOM_V,
            state.zoom_v,
            state.run_time,
            read.prev + 80,
        )?;

        Ok(Self {
            name,
            clip_near,
            clip_far,
            lod_multiplier,
            fov_h,
            fov_v,
            zoom_h,
            zoom_v,
            run_time: state.run_time,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let node_index = anim_def.node_to_index(&self.name)?;
        let mut flags = CameraStateFlags::empty();

        if self.clip_near.is_some() {
            flags |= CameraStateFlags::CLIP_NEAR;
        }
        if self.clip_far.is_some() {
            flags |= CameraStateFlags::CLIP_FAR;
        }
        if self.lod_multiplier.is_some() {
            flags |= CameraStateFlags::LOD_MULTIPLIER;
        }
        if self.fov_h.is_some() {
            flags |= CameraStateFlags::FOV_H;
        }
        if self.fov_v.is_some() {
            flags |= CameraStateFlags::FOV_V;
        }
        if self.zoom_h.is_some() {
            flags |= CameraStateFlags::ZOOM_H;
        }
        if self.zoom_v.is_some() {
            flags |= CameraStateFlags::ZOOM_V;
        }

        let state = CameraFromToC {
            flags: flags.maybe(),
            node_index,
            clip_near: make_value(&self.clip_near, self.run_time),
            clip_far: make_value(&self.clip_far, self.run_time),
            lod_multiplier: make_value(&self.lod_multiplier, self.run_time),
            fov_h: make_value(&self.fov_h, self.run_time),
            fov_v: make_value(&self.fov_v, self.run_time),
            zoom_h: make_value(&self.zoom_h, self.run_time),
            zoom_v: make_value(&self.zoom_v, self.run_time),
            run_time: self.run_time,
        };
        write.write_struct(&state)?;
        Ok(())
    }
}
