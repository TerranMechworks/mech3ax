use super::EventAll;
use super::delta::delta;
use crate::types::{AnimDefLookup as _, Idx32};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::Vec3;
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::anim::events::{FloatFromTo, ObjectMotionFromTo, Vec3FromTo};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{Result, assert_that};
use mech3ax_types::{AsBytes as _, Maybe, bitflags, impl_as_bytes};
use std::io::{Read, Write};

bitflags! {
    struct ObjectMotionFromToFlags: u32 {
        const TRANSLATE = 1 << 0;   // 0x1
        const ROTATE = 1 << 1;      // 0x2
        const SCALE = 1 << 2;       // 0x4
        const MORPH = 1 << 3;       // 0x8
    }
}

type Flags = Maybe<u32, ObjectMotionFromToFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ObjectMotionFromToC {
    flags: Flags,          // 000
    node_index: Idx32,     // 004
    morph_from: f32,       // 008
    morph_to: f32,         // 012
    morph_delta: f32,      // 016
    translate_from: Vec3,  // 020
    translate_to: Vec3,    // 032
    translate_delta: Vec3, // 044
    rotate_from: Vec3,     // 056
    rotate_to: Vec3,       // 068
    rotate_delta: Vec3,    // 080
    scale_from: Vec3,      // 092
    scale_to: Vec3,        // 104
    scale_delta: Vec3,     // 116
    run_time: f32,         // 128
}
impl_as_bytes!(ObjectMotionFromToC, 132);

fn delta_vec3(from: &Vec3, to: &Vec3, run_time: f32) -> Vec3 {
    Vec3 {
        x: delta(from.x, to.x, run_time),
        y: delta(from.y, to.y, run_time),
        z: delta(from.z, to.z, run_time),
    }
}

impl EventAll for ObjectMotionFromTo {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(ObjectMotionFromToC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!(
            "object motion from to size",
            size == ObjectMotionFromToC::SIZE,
            read.offset
        )?;
        let motion: ObjectMotionFromToC = read.read_struct()?;

        assert_that!(
            "object motion from to run time",
            motion.run_time > 0.0,
            read.prev + 128
        )?;
        let run_time = motion.run_time;

        let flags = assert_that!("object motion from to flags", flags motion.flags, read.prev + 0)?;
        let name = anim_def.node_from_index(motion.node_index, read.prev + 4)?;

        let morph = if flags.contains(ObjectMotionFromToFlags::MORPH) {
            assert_that!(
                "object motion from to morph from",
                0.0 <= motion.morph_from <= 1.0,
                read.prev + 8
            )?;
            assert_that!(
                "object motion from to morph to",
                0.0 <= motion.morph_to <= 1.0,
                read.prev + 12
            )?;
            let morph_delta = delta(motion.morph_from, motion.morph_to, run_time);
            assert_that!(
                "object motion from to morph delta",
                motion.morph_delta == morph_delta,
                read.prev + 16
            )?;
            Some(FloatFromTo {
                from: motion.morph_from,
                to: motion.morph_to,
            })
        } else {
            assert_that!(
                "object motion from to morph from",
                motion.morph_from == 0.0,
                read.prev + 8
            )?;
            assert_that!(
                "object motion from to morph to",
                motion.morph_to == 0.0,
                read.prev + 12
            )?;
            assert_that!(
                "object motion from to morph delta",
                motion.morph_delta == 0.0,
                read.prev + 16
            )?;
            None
        };

        let mut translate_delta = None;
        let translate = if flags.contains(ObjectMotionFromToFlags::TRANSLATE) {
            let delta = delta_vec3(&motion.translate_from, &motion.translate_to, run_time);
            if motion.translate_delta == delta {
                log::trace!("object motion from to translate delta: OK");
            } else {
                log::debug!("object motion from to translate delta: FAIL");
                translate_delta = Some(motion.translate_delta);
            }

            Some(Vec3FromTo {
                from: motion.translate_from,
                to: motion.translate_to,
            })
        } else {
            assert_that!(
                "object motion from to translate from",
                motion.translate_from == Vec3::DEFAULT,
                read.prev + 20
            )?;
            assert_that!(
                "object motion from to translate to",
                motion.translate_to == Vec3::DEFAULT,
                read.prev + 32
            )?;
            assert_that!(
                "object motion from to translate delta",
                motion.translate_delta == Vec3::DEFAULT,
                read.prev + 44
            )?;
            None
        };

        let mut rotate_delta = None;
        let rotate: Option<Vec3FromTo> = if flags.contains(ObjectMotionFromToFlags::ROTATE) {
            let delta = delta_vec3(&motion.rotate_from, &motion.rotate_to, run_time);
            if motion.rotate_delta == delta {
                log::trace!("object motion from to rotate delta: OK");
            } else {
                log::debug!("object motion from to rotate delta: FAIL");
                rotate_delta = Some(motion.rotate_delta);
            }

            Some(Vec3FromTo {
                from: motion.rotate_from,
                to: motion.rotate_to,
            })
        } else {
            assert_that!(
                "object motion rotate from",
                motion.rotate_from == Vec3::DEFAULT,
                read.prev + 56
            )?;
            assert_that!(
                "object motion rotate to",
                motion.rotate_to == Vec3::DEFAULT,
                read.prev + 68
            )?;
            assert_that!(
                "object motion rotate delta",
                motion.rotate_delta == Vec3::DEFAULT,
                read.prev + 80
            )?;
            None
        };

        let mut scale_delta = None;
        let scale = if flags.contains(ObjectMotionFromToFlags::SCALE) {
            let delta = delta_vec3(&motion.scale_from, &motion.scale_to, run_time);
            if motion.scale_delta == delta {
                log::trace!("object motion from to scale delta: OK");
            } else {
                log::debug!("object motion from to scale delta: FAIL");
                scale_delta = Some(motion.scale_delta);
            }

            Some(Vec3FromTo {
                from: motion.scale_from,
                to: motion.scale_to,
            })
        } else {
            assert_that!(
                "object motion from to scale from",
                motion.scale_from == Vec3::DEFAULT,
                read.prev + 92
            )?;
            assert_that!(
                "object motion from to scale to",
                motion.scale_to == Vec3::DEFAULT,
                read.prev + 104
            )?;
            assert_that!(
                "object motion from to scale delta",
                motion.scale_delta == Vec3::DEFAULT,
                read.prev + 116
            )?;
            None
        };

        Ok(Self {
            name,
            run_time,
            morph,
            translate,
            rotate,
            scale,
            translate_delta,
            rotate_delta,
            scale_delta,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let mut flags = ObjectMotionFromToFlags::empty();
        if self.translate.is_some() {
            flags |= ObjectMotionFromToFlags::TRANSLATE;
        }
        if self.rotate.is_some() {
            flags |= ObjectMotionFromToFlags::ROTATE;
        }
        if self.scale.is_some() {
            flags |= ObjectMotionFromToFlags::SCALE;
        }
        if self.morph.is_some() {
            flags |= ObjectMotionFromToFlags::MORPH;
        }

        let node_index = anim_def.node_to_index(&self.name)?;
        let run_time = self.run_time;

        let (morph_from, morph_to) = self
            .morph
            .as_ref()
            .map(|morph| (morph.from, morph.to))
            .unwrap_or((0.0, 0.0));
        let morph_delta = delta(morph_from, morph_to, run_time);

        let (translate_from, translate_to) = self
            .translate
            .as_ref()
            .map(|translate| (translate.from, translate.to))
            .unwrap_or((Vec3::DEFAULT, Vec3::DEFAULT));
        let translate_delta = self
            .translate_delta
            .unwrap_or_else(|| delta_vec3(&translate_from, &translate_to, run_time));

        let (rotate_from, rotate_to) = self
            .rotate
            .as_ref()
            .map(|rotate| (rotate.from, rotate.to))
            .unwrap_or((Vec3::DEFAULT, Vec3::DEFAULT));
        let rotate_delta = self
            .rotate_delta
            .unwrap_or_else(|| delta_vec3(&rotate_from, &rotate_to, run_time));

        let (scale_from, scale_to) = self
            .scale
            .as_ref()
            .map(|scale| (scale.from, scale.to))
            .unwrap_or((Vec3::DEFAULT, Vec3::DEFAULT));
        let scale_delta = self
            .scale_delta
            .unwrap_or_else(|| delta_vec3(&scale_from, &scale_to, run_time));

        let motion = ObjectMotionFromToC {
            flags: flags.maybe(),
            node_index,
            morph_from,
            morph_to,
            morph_delta,
            translate_from,
            translate_to,
            translate_delta,
            rotate_from,
            rotate_to,
            rotate_delta,
            scale_from,
            scale_to,
            scale_delta,
            run_time,
        };
        write.write_struct(&motion)?;
        Ok(())
    }
}
