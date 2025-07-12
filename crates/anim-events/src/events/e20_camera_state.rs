use super::EventAll;
use crate::types::{AnimDefLookup as _, Idx32};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::anim::events::CameraState;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{Result, assert_that};
use mech3ax_types::{AsBytes as _, Maybe, bitflags, impl_as_bytes};
use std::io::{Read, Write};

bitflags! {
    pub(crate) struct CameraStateFlags: u32 {
        const CLIP_NEAR = 1 << 0;           // 0x01
        const CLIP_FAR = 1 << 1;            // 0x02
        const LOD_MULTIPLIER = 1 << 2;      // 0x04
        const FOV_H = 1 << 3;               // 0x08
        const FOV_V = 1 << 4;               // 0x10
        const ZOOM_H = 1 << 5;              // 0x20
        const ZOOM_V = 1 << 6;              // 0x40
    }
}

type Flags = Maybe<u32, CameraStateFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct CameraStateC {
    flags: Flags,        // 00
    node_index: Idx32,   // 04
    clip_near: f32,      // 08
    clip_far: f32,       // 12
    lod_multiplier: f32, // 16
    fov_h: f32,          // 20
    fov_v: f32,          // 24
    zoom_h: f32,         // 28
    zoom_v: f32,         // 32
}
impl_as_bytes!(CameraStateC, 36);

fn assert_flag_and_f32(
    name: &str,
    flags: CameraStateFlags,
    flag: CameraStateFlags,
    value: f32,
    offset: usize,
) -> Result<Option<f32>> {
    if flags.contains(flag) {
        Ok(Some(value))
    } else {
        assert_that!(name, value == 0.0, offset)?;
        Ok(None)
    }
}

impl EventAll for CameraState {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(CameraStateC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("camera state size", size == CameraStateC::SIZE, read.offset)?;
        let state: CameraStateC = read.read_struct()?;

        let flags = assert_that!("camera state flags", flags state.flags, read.prev + 0)?;
        let name = anim_def.node_from_index(state.node_index, read.prev + 4)?;

        let clip_near = assert_flag_and_f32(
            "camera state clip near",
            flags,
            CameraStateFlags::CLIP_NEAR,
            state.clip_near,
            read.prev + 8,
        )?;
        let clip_far = assert_flag_and_f32(
            "camera state clip far",
            flags,
            CameraStateFlags::CLIP_FAR,
            state.clip_far,
            read.prev + 12,
        )?;
        let lod_multiplier = assert_flag_and_f32(
            "camera state lod multiplier",
            flags,
            CameraStateFlags::LOD_MULTIPLIER,
            state.lod_multiplier,
            read.prev + 16,
        )?;
        let fov_h = assert_flag_and_f32(
            "camera state fov h",
            flags,
            CameraStateFlags::FOV_H,
            state.fov_h,
            read.prev + 20,
        )?;
        let fov_v = assert_flag_and_f32(
            "camera state fov v",
            flags,
            CameraStateFlags::FOV_V,
            state.fov_v,
            read.prev + 24,
        )?;
        let zoom_h = assert_flag_and_f32(
            "camera state zoom h",
            flags,
            CameraStateFlags::ZOOM_H,
            state.zoom_h,
            read.prev + 28,
        )?;
        let zoom_v = assert_flag_and_f32(
            "camera state fov zoom v",
            flags,
            CameraStateFlags::ZOOM_V,
            state.zoom_v,
            read.prev + 32,
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

        let state = CameraStateC {
            flags: flags.maybe(),
            node_index,
            clip_near: self.clip_near.unwrap_or(0.0),
            clip_far: self.clip_far.unwrap_or(0.0),
            lod_multiplier: self.lod_multiplier.unwrap_or(0.0),
            fov_h: self.fov_h.unwrap_or(0.0),
            fov_v: self.fov_v.unwrap_or(0.0),
            zoom_h: self.zoom_h.unwrap_or(0.0),
            zoom_v: self.zoom_v.unwrap_or(0.0),
        };
        write.write_struct(&state)?;
        Ok(())
    }
}
