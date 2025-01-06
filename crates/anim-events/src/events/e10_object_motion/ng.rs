use super::{EventMw, EventPm, Flags, ObjectMotionFlags};
use crate::types::{index, AnimDefLookup as _, Idx16};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::{
    BounceSequences, BounceSound, BounceSounds, ForwardRotation, ForwardRotationDistance,
    ForwardRotationTime, Gravity, ObjectMotion, ObjectMotionScale, ObjectMotionTranslation,
    ObjectMotionXyzRot, TranslationRange,
};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::{Range, Vec3};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{impl_as_bytes, AsBytes as _, Ascii};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ObjectMotionNgC {
    flags: Flags,                // 000
    node_index: Idx16,           // 004
    pad006: u16,                 // 006
    zero008: f32,                // 008
    gravity: f32,                // 012
    morph: f32,                  // 016
    trans_range_xz: Range,       // 020
    trans_range_y: Range,        // 028
    trans_range_initial: Range,  // 036
    trans_range_delta: Range,    // 044
    trans_initial: Vec3,         // 052
    trans_delta: Vec3,           // 064
    trans_current: Vec3,         // 076
    trans_delta_grav: Vec3,      // 088
    trans_rnd_xz: Vec3,          // 100
    fwd_rot_initial: f32,        // 112
    fwd_rot_delta: f32,          // 116
    fwd_rot_current: f32,        // 120
    xyz_rot_initial: Vec3,       // 124
    xyz_rot_delta: Vec3,         // 136
    xyz_rot_current: Vec3,       // 148
    scale_initial: Vec3,         // 160
    scale_delta: Vec3,           // 172
    scale_current: Vec3,         // 184
    bounce_seq0_name: Ascii<32>, // 196, default
    bounce_seq0_index: i16,      // 228
    bounce_snd0_index: Idx16,    // 230
    bounce_snd0_volume: f32,     // 232
    bounce_seq1_name: Ascii<32>, // 236, water
    bounce_seq1_index: i16,      // 268
    bounce_snd1_index: Idx16,    // 270
    bounce_snd1_volume: f32,     // 272
    bounce_seq2_name: Ascii<32>, // 276, lava
    bounce_seq2_index: i16,      // 308
    bounce_snd2_index: Idx16,    // 310
    bounce_snd2_volume: f32,     // 312
    run_time: f32,               // 316
}
impl_as_bytes!(ObjectMotionNgC, 320);

fn object_flags_is_borked(anim_def: &AnimDef) -> bool {
    // these anim defs in MW C4 lack TRANSLATION_RANGE_MAX
    (anim_def.name == "ramp1" || anim_def.name == "ramp2")
        && (anim_def.anim_name == "hit_ramp1" || anim_def.anim_name == "hit_ramp2")
}

impl EventMw for ObjectMotion {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(ObjectMotionNgC::SIZE)
    }

    #[inline]
    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        read_ng(read, anim_def, size)
    }

    #[inline]
    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        write_ng(self, write, anim_def)
    }
}

impl EventPm for ObjectMotion {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(ObjectMotionNgC::SIZE)
    }

    #[inline]
    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        read_ng(read, anim_def, size)
    }

    #[inline]
    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        write_ng(self, write, anim_def)
    }
}

fn read_ng(
    read: &mut CountingReader<impl Read>,
    anim_def: &AnimDef,
    size: u32,
) -> Result<ObjectMotion> {
    assert_that!(
        "object motion size",
        size == ObjectMotionNgC::SIZE,
        read.offset
    )?;
    let motion: ObjectMotionNgC = read.read_struct()?;

    let mut flags = assert_that!("object motion flags", flags motion.flags, read.prev + 0)?;
    let node = anim_def.node_from_index(motion.node_index, read.prev + 4)?;

    assert_that!("object motion field 006", motion.pad006 == 0, read.prev + 6)?;
    assert_that!(
        "object motion field 008",
        motion.zero008 == 0.0,
        read.prev + 8
    )?;

    let morph = if flags.contains(ObjectMotionFlags::MORPH) {
        Some(motion.morph)
    } else {
        assert_that!("object motion morph", motion.morph == 0.0, read.prev + 16)?;
        None
    };

    let gravity_complex = flags.contains(ObjectMotionFlags::GRAVITY_COMPLEX);
    let gravity_no_altitude = flags.contains(ObjectMotionFlags::GRAVITY_NO_ALTITUDE);

    let gravity = if flags.contains(ObjectMotionFlags::GRAVITY) {
        Some(Gravity {
            value: motion.gravity,
            complex: gravity_complex,
            no_altitude: gravity_no_altitude,
        })
    } else {
        assert_that!(
            "object motion gravity complex",
            gravity_complex == false,
            read.prev + 0
        )?;
        assert_that!(
            "object motion gravity no altitude",
            gravity_no_altitude == false,
            read.prev + 0
        )?;
        assert_that!(
            "object motion gravity value",
            motion.gravity == 0.0,
            read.prev + 12
        )?;
        None
    };

    let translation_range_min = flags.contains(ObjectMotionFlags::TRANSLATION_RANGE_MIN);
    let translation_range_max = flags.contains(ObjectMotionFlags::TRANSLATION_RANGE_MAX);

    if object_flags_is_borked(anim_def) && translation_range_min && !translation_range_max {
        log::debug!("ObjectMotion flags translation range fixup");
        flags |= ObjectMotionFlags::TRANSLATION_RANGE_MAX;
    }

    let translation_range = if flags.contains(ObjectMotionFlags::TRANSLATION_RANGE) {
        Some(TranslationRange {
            xz: motion.trans_range_xz,
            y: motion.trans_range_y,
            initial: motion.trans_range_initial,
            delta: motion.trans_range_delta,
        })
    } else {
        assert_that!(
            "object motion flags translation range min",
            translation_range_min == false,
            read.prev + 0
        )?;
        assert_that!(
            "object motion flags translation range max",
            translation_range_max == false,
            read.prev + 0
        )?;

        assert_that!(
            "object motion trans range xz",
            motion.trans_range_xz == Range::DEFAULT,
            read.prev + 20
        )?;
        assert_that!(
            "object motion trans range y",
            motion.trans_range_y == Range::DEFAULT,
            read.prev + 28
        )?;
        assert_that!(
            "object motion trans range initial",
            motion.trans_range_initial == Range::DEFAULT,
            read.prev + 36
        )?;
        assert_that!(
            "object motion trans range delta",
            motion.trans_range_delta == Range::DEFAULT,
            read.prev + 44
        )?;
        None
    };

    let translation = if flags.contains(ObjectMotionFlags::TRANSLATION) {
        Some(ObjectMotionTranslation {
            initial: motion.trans_initial,
            delta: motion.trans_delta,
            rnd_xz: motion.trans_rnd_xz,
        })
    } else {
        assert_that!(
            "object motion trans initial",
            motion.trans_initial == Vec3::DEFAULT,
            read.prev + 52
        )?;
        assert_that!(
            "object motion trans delta",
            motion.trans_delta == Vec3::DEFAULT,
            read.prev + 64
        )?;
        assert_that!(
            "object motion trans rnd xz",
            motion.trans_rnd_xz == Vec3::DEFAULT,
            read.prev + 100
        )?;
        None
    };
    assert_that!(
        "object motion trans current",
        motion.trans_current == Vec3::DEFAULT,
        read.prev + 76
    )?;
    assert_that!(
        "object motion trans grav",
        motion.trans_delta_grav == Vec3::DEFAULT,
        read.prev + 88
    )?;

    let forward_rotation_time = flags.contains(ObjectMotionFlags::FORWARD_ROTATION_TIME);
    let forward_rotation_dist = flags.contains(ObjectMotionFlags::FORWARD_ROTATION_DISTANCE);

    let forward_rotation = if forward_rotation_time {
        assert_that!(
            "object motion fwd rot dist",
            forward_rotation_dist == false,
            read.prev + 0
        )?;
        Some(ForwardRotation::Time(ForwardRotationTime {
            initial: motion.fwd_rot_initial,
            delta: motion.fwd_rot_delta,
        }))
    } else if forward_rotation_dist {
        assert_that!(
            "object motion fwd rot delta",
            motion.fwd_rot_delta == 0.0,
            read.prev + 116
        )?;
        Some(ForwardRotation::Distance(ForwardRotationDistance {
            initial: motion.fwd_rot_initial,
        }))
    } else {
        assert_that!(
            "object motion fwd rot initial",
            motion.fwd_rot_initial == 0.0,
            read.prev + 112
        )?;
        assert_that!(
            "object motion fwd rot delta",
            motion.fwd_rot_delta == 0.0,
            read.prev + 116
        )?;
        None
    };
    assert_that!(
        "object motion fwd rot current",
        motion.fwd_rot_current == 0.0,
        read.prev + 120
    )?;

    let xyz_rotation = if flags.contains(ObjectMotionFlags::XYZ_ROTATION) {
        Some(ObjectMotionXyzRot {
            initial: motion.xyz_rot_initial,
            delta: motion.xyz_rot_delta,
        })
    } else {
        assert_that!(
            "object motion xyz rot initial",
            motion.xyz_rot_initial == Vec3::DEFAULT,
            read.prev + 124
        )?;
        assert_that!(
            "object motion xyz rot delta",
            motion.xyz_rot_delta == Vec3::DEFAULT,
            read.prev + 136
        )?;
        None
    };
    assert_that!(
        "object motion xyz rot current",
        motion.xyz_rot_current == Vec3::DEFAULT,
        read.prev + 148
    )?;

    let scale = if flags.contains(ObjectMotionFlags::SCALE) {
        Some(ObjectMotionScale {
            initial: motion.scale_initial,
            delta: motion.scale_delta,
        })
    } else {
        assert_that!(
            "object motion scale initial",
            motion.scale_initial == Vec3::DEFAULT,
            read.prev + 160
        )?;
        assert_that!(
            "object motion scale delta",
            motion.scale_delta == Vec3::DEFAULT,
            read.prev + 172
        )?;
        None
    };
    assert_that!(
        "object motion scale current",
        motion.scale_current == Vec3::DEFAULT,
        read.prev + 184
    )?;

    assert_that!(
        "object motion bounce seq 0 index",
        motion.bounce_seq0_index == -1,
        read.prev + 228
    )?;
    assert_that!(
        "object motion bounce seq 1 index",
        motion.bounce_seq1_index == -1,
        read.prev + 268
    )?;
    assert_that!(
        "object motion bounce seq 2 index",
        motion.bounce_seq2_index == -1,
        read.prev + 308
    )?;

    let bounce_sequence = if flags.contains(ObjectMotionFlags::BOUNCE_SEQUENCE) {
        let seq_name0 = if !motion.bounce_seq0_name.first_is_zero() {
            let bounce_seq0 =
                assert_utf8("object motion bounce seq 0 name", read.prev + 196, || {
                    motion.bounce_seq0_name.to_str_padded()
                })?;
            Some(bounce_seq0)
        } else {
            assert_that!("object motion bounce seq 0", zero motion.bounce_seq0_name, read.prev + 196)?;
            None
        };

        let seq_name1 = if !motion.bounce_seq1_name.first_is_zero() {
            let bounce_seq1 =
                assert_utf8("object motion bounce seq 1 name", read.prev + 236, || {
                    motion.bounce_seq1_name.to_str_padded()
                })?;
            Some(bounce_seq1)
        } else {
            assert_that!("object motion bounce seq 1", zero motion.bounce_seq1_name, read.prev + 236)?;
            None
        };

        let seq_name2 = if !motion.bounce_seq2_name.first_is_zero() {
            let bounce_seq2 =
                assert_utf8("object motion bounce seq 2 name", read.prev + 276, || {
                    motion.bounce_seq2_name.to_str_padded()
                })?;
            Some(bounce_seq2)
        } else {
            assert_that!("object motion bounce seq 2", zero motion.bounce_seq2_name, read.prev + 276)?;
            None
        };

        Some(BounceSequences {
            default: seq_name0,
            water: seq_name1,
            lava: seq_name2,
        })
    } else {
        assert_that!("object motion bounce seq 0", zero motion.bounce_seq0_name, read.prev + 196)?;
        assert_that!("object motion bounce seq 1", zero motion.bounce_seq1_name, read.prev + 236)?;
        assert_that!("object motion bounce seq 2", zero motion.bounce_seq2_name, read.prev + 276)?;
        None
    };

    let bounce_sound = if flags.contains(ObjectMotionFlags::BOUNCE_SOUND) {
        let snd0 = if motion.bounce_snd0_index > 0 {
            assert_that!(
                "object motion bounce snd 0 vol",
                motion.bounce_snd0_volume > 0.0,
                read.prev + 232
            )?;
            let sound_name =
                anim_def.stc_sound_from_index(motion.bounce_snd0_index, read.prev + 230)?;

            Some(BounceSound {
                name: sound_name,
                volume: motion.bounce_snd0_volume,
            })
        } else {
            None
        };

        let snd1 = if motion.bounce_snd1_index > 0 {
            assert_that!(
                "object motion bounce snd 1 vol",
                motion.bounce_snd1_volume > 0.0,
                read.prev + 272
            )?;
            let sound_name =
                anim_def.stc_sound_from_index(motion.bounce_snd1_index, read.prev + 270)?;

            Some(BounceSound {
                name: sound_name,
                volume: motion.bounce_snd1_volume,
            })
        } else {
            None
        };

        let snd2 = if motion.bounce_snd2_index > 0 {
            assert_that!(
                "object motion bounce snd 2 vol",
                motion.bounce_snd2_volume > 0.0,
                read.prev + 312
            )?;
            let sound_name =
                anim_def.stc_sound_from_index(motion.bounce_snd2_index, read.prev + 310)?;

            Some(BounceSound {
                name: sound_name,
                volume: motion.bounce_snd2_volume,
            })
        } else {
            None
        };

        Some(BounceSounds {
            default: snd0,
            water: snd1,
            lava: snd2,
        })
    } else {
        assert_that!(
            "object motion bounce snd 0 index",
            motion.bounce_snd0_index == index!(0),
            read.prev + 230
        )?;
        assert_that!(
            "object motion bounce snd 0 vol",
            motion.bounce_snd0_volume == 0.0,
            read.prev + 232
        )?;
        assert_that!(
            "object motion bounce snd 1 index",
            motion.bounce_snd1_index == index!(0),
            read.prev + 270
        )?;
        assert_that!(
            "object motion bounce snd 1 vol",
            motion.bounce_snd1_volume == 0.0,
            read.prev + 272
        )?;
        assert_that!(
            "object motion bounce snd 2 index",
            motion.bounce_snd2_index == index!(0),
            read.prev + 310
        )?;
        assert_that!(
            "object motion bounce snd 2 vol",
            motion.bounce_snd2_volume == 0.0,
            read.prev + 312
        )?;
        None
    };

    let run_time = if flags.contains(ObjectMotionFlags::RUN_TIME) {
        assert_that!(
            "object motion run time",
            motion.run_time > 0.0,
            read.prev + 316
        )?;
        Some(motion.run_time)
    } else {
        assert_that!(
            "object motion run time",
            motion.run_time == 0.0,
            read.prev + 316
        )?;
        None
    };

    Ok(ObjectMotion {
        node,
        impact_force: flags.contains(ObjectMotionFlags::IMPACT_FORCE),
        morph,
        gravity,
        translation_range,
        translation,
        forward_rotation,
        xyz_rotation,
        scale,
        bounce_sequence,
        bounce_sound,
        run_time,
    })
}

fn write_ng(
    motion: &ObjectMotion,
    write: &mut CountingWriter<impl Write>,
    anim_def: &AnimDef,
) -> Result<()> {
    let node_index = anim_def.node_to_index(&motion.node)?;
    let mut flags = ObjectMotionFlags::empty();

    if motion.impact_force {
        flags |= ObjectMotionFlags::IMPACT_FORCE;
    }

    let morph = if let Some(morph) = motion.morph {
        flags |= ObjectMotionFlags::MORPH;
        morph
    } else {
        0.0
    };

    let gravity = if let Some(gravity) = &motion.gravity {
        flags |= ObjectMotionFlags::GRAVITY;
        if gravity.complex {
            flags |= ObjectMotionFlags::GRAVITY_COMPLEX;
        }
        if gravity.no_altitude {
            flags |= ObjectMotionFlags::GRAVITY_NO_ALTITUDE;
        }
        gravity.value
    } else {
        0.0
    };

    let mut trans_range_xz = Range::DEFAULT;
    let mut trans_range_y = Range::DEFAULT;
    let mut trans_range_initial = Range::DEFAULT;
    let mut trans_range_delta = Range::DEFAULT;

    if let Some(tr) = &motion.translation_range {
        if object_flags_is_borked(anim_def) {
            log::debug!("ObjectMotion flags translation range fixup");
            flags |= ObjectMotionFlags::TRANSLATION_RANGE_MIN;
        } else {
            flags |= ObjectMotionFlags::TRANSLATION_RANGE;
        }

        trans_range_xz = tr.xz;
        trans_range_y = tr.y;
        trans_range_initial = tr.initial;
        trans_range_delta = tr.delta;
    }

    let mut trans_initial = Vec3::DEFAULT;
    let mut trans_delta = Vec3::DEFAULT;
    let mut trans_rnd_xz = Vec3::DEFAULT;

    if let Some(tr) = &motion.translation {
        flags |= ObjectMotionFlags::TRANSLATION;

        trans_initial = tr.initial;
        trans_delta = tr.delta;
        trans_rnd_xz = tr.rnd_xz;
    }

    let (fwd_rot_initial, fwd_rot_delta) = match &motion.forward_rotation {
        Some(ForwardRotation::Time(time)) => {
            flags |= ObjectMotionFlags::FORWARD_ROTATION_TIME;
            (time.initial, time.delta)
        }
        Some(ForwardRotation::Distance(dist)) => {
            flags |= ObjectMotionFlags::FORWARD_ROTATION_DISTANCE;
            (dist.initial, 0.0)
        }
        None => (0.0, 0.0),
    };

    let (xyz_rot_initial, xyz_rot_delta) = motion
        .xyz_rotation
        .clone()
        .map(|ObjectMotionXyzRot { initial, delta }| {
            flags |= ObjectMotionFlags::XYZ_ROTATION;
            (initial, delta)
        })
        .unwrap_or_else(|| (Vec3::DEFAULT, Vec3::DEFAULT));

    let (scale_initial, scale_delta) = motion
        .scale
        .clone()
        .map(|ObjectMotionScale { initial, delta }| {
            flags |= ObjectMotionFlags::SCALE;
            (initial, delta)
        })
        .unwrap_or_else(|| (Vec3::DEFAULT, Vec3::DEFAULT));

    let mut bounce_seq0_name = Ascii::zero();
    let mut bounce_seq1_name = Ascii::zero();
    let mut bounce_seq2_name = Ascii::zero();

    if let Some(bounce_seq) = &motion.bounce_sequence {
        flags |= ObjectMotionFlags::BOUNCE_SEQUENCE;

        if let Some(s) = &bounce_seq.default {
            bounce_seq0_name = Ascii::from_str_padded(s);
        }
        if let Some(s) = &bounce_seq.water {
            bounce_seq1_name = Ascii::from_str_padded(s);
        }
        if let Some(s) = &bounce_seq.lava {
            bounce_seq2_name = Ascii::from_str_padded(s);
        }
    }

    let mut bounce_snd0_index = index!(0);
    let mut bounce_snd0_volume = 0.0;
    let mut bounce_snd1_index = index!(0);
    let mut bounce_snd1_volume = 0.0;
    let mut bounce_snd2_index = index!(0);
    let mut bounce_snd2_volume = 0.0;

    if let Some(bounce) = &motion.bounce_sound {
        flags |= ObjectMotionFlags::BOUNCE_SOUND;

        if let Some(s) = &bounce.default {
            bounce_snd0_index = anim_def.stc_sound_to_index(&s.name)?;
            bounce_snd0_volume = s.volume;
        }
        if let Some(s) = &bounce.water {
            bounce_snd1_index = anim_def.stc_sound_to_index(&s.name)?;
            bounce_snd1_volume = s.volume;
        }
        if let Some(s) = &bounce.lava {
            bounce_snd2_index = anim_def.stc_sound_to_index(&s.name)?;
            bounce_snd2_volume = s.volume;
        }
    };

    if motion.run_time.is_some() {
        flags |= ObjectMotionFlags::RUN_TIME;
    }
    let run_time = motion.run_time.unwrap_or(0.0);

    let object_motion = ObjectMotionNgC {
        flags: flags.maybe(),
        node_index,
        pad006: 0,
        zero008: 0.0,
        gravity,
        morph,
        trans_range_xz,
        trans_range_y,
        trans_range_initial,
        trans_range_delta,
        trans_initial,
        trans_delta,
        trans_current: Vec3::DEFAULT,
        trans_delta_grav: Vec3::DEFAULT,
        trans_rnd_xz,
        fwd_rot_initial,
        fwd_rot_delta,
        fwd_rot_current: 0.0,
        xyz_rot_initial,
        xyz_rot_delta,
        xyz_rot_current: Vec3::DEFAULT,
        scale_initial,
        scale_delta,
        scale_current: Vec3::DEFAULT,
        bounce_seq0_name,
        bounce_seq0_index: -1,
        bounce_snd0_index,
        bounce_snd0_volume,
        bounce_seq1_name,
        bounce_seq1_index: -1,
        bounce_snd1_index,
        bounce_snd1_volume,
        bounce_seq2_name,
        bounce_seq2_index: -1,
        bounce_snd2_index,
        bounce_snd2_volume,
        run_time,
    };
    write.write_struct(&object_motion)?;
    Ok(())
}
