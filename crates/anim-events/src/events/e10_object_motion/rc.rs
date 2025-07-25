use super::{EventRc, Flags, ObjectMotionFlags};
use crate::types::{AnimDefLookup as _, Idx16, index};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::anim::events::{
    BounceSequences, BounceSound, BounceSounds, ForwardRotation, ForwardRotationDistance,
    ForwardRotationTime, Gravity, ObjectMotion, ObjectMotionScale, ObjectMotionTranslation,
    ObjectMotionXyzRot, TranslationRange,
};
use mech3ax_api_types::{Range, Vec3};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{Result, assert_that, assert_with_msg};
use mech3ax_types::{AsBytes as _, Ascii, impl_as_bytes};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ObjectMotionRcC {
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
    bounce_seq0_name: Ascii<32>, // 196
    bounce_seq0_index: i16,      // 228
    bounce_snd0_index: Idx16,    // 230
    bounce_snd0_volume: f32,     // 232
    run_time: f32,               // 236
}
impl_as_bytes!(ObjectMotionRcC, 240);

impl EventRc for ObjectMotion {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(ObjectMotionRcC::SIZE)
    }

    #[inline]
    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        read_pg(read, anim_def, size)
    }

    #[inline]
    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        write_pg(self, write, anim_def)
    }
}

fn read_pg(
    read: &mut CountingReader<impl Read>,
    anim_def: &AnimDef,
    size: u32,
) -> Result<ObjectMotion> {
    assert_that!(
        "object motion size",
        size == ObjectMotionRcC::SIZE,
        read.offset
    )?;
    let motion: ObjectMotionRcC = read.read_struct()?;

    let flags = assert_that!("object motion flags", flags motion.flags, read.prev + 0)?;
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
        // no altitude gravity is not supported
        assert_that!(
            "object motion gravity no altitude",
            gravity_no_altitude == false,
            read.prev + 0
        )?;
        Some(Gravity {
            value: motion.gravity,
            complex: gravity_complex,
            no_altitude: false,
        })
    } else {
        assert_that!(
            "object motion gravity complex",
            gravity_complex == false,
            read.prev + 0
        )?;
        // no altitude gravity is not supported
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

    let translation_range = if flags.contains(ObjectMotionFlags::TRANSLATION_RANGE) {
        Some(TranslationRange {
            xz: motion.trans_range_xz,
            y: motion.trans_range_y,
            initial: motion.trans_range_initial,
            delta: motion.trans_range_delta,
        })
    } else {
        let translation_range_min = flags.contains(ObjectMotionFlags::TRANSLATION_RANGE_MIN);
        let translation_range_max = flags.contains(ObjectMotionFlags::TRANSLATION_RANGE_MAX);
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

    let bounce_sequence = if flags.contains(ObjectMotionFlags::BOUNCE_SEQUENCE) {
        let seq_name0 = assert_utf8(
            "object motion bounce sequence name",
            read.prev + 196,
            || motion.bounce_seq0_name.to_str_padded(),
        )?;
        assert_that!(
            "object motion bounce sequence index",
            motion.bounce_seq0_index == -1,
            read.prev + 228
        )?;
        Some(BounceSequences {
            default: Some(seq_name0),
            water: None,
            lava: None,
        })
    } else {
        assert_that!("object motion bounce sequence name", zero motion.bounce_seq0_name, read.prev + 196)?;
        assert_that!(
            "object motion bounce sequence index",
            motion.bounce_seq0_index == 0,
            read.prev + 228
        )?;
        None
    };

    let bounce_sound = if flags.contains(ObjectMotionFlags::BOUNCE_SOUND) {
        let name = anim_def.stc_sound_from_index(motion.bounce_snd0_index, read.prev + 230)?;
        let snd0 = BounceSound {
            name,
            volume: motion.bounce_snd0_volume,
        };

        Some(BounceSounds {
            default: Some(snd0),
            water: None,
            lava: None,
        })
    } else {
        assert_that!(
            "object motion bounce sound index",
            motion.bounce_snd0_index == index!(0),
            read.prev + 230
        )?;
        assert_that!(
            "object motion bounce sound volume",
            motion.bounce_snd0_volume == 0.0,
            read.prev + 232
        )?;
        None
    };

    let run_time = if flags.contains(ObjectMotionFlags::RUN_TIME) {
        assert_that!(
            "object motion run time",
            motion.run_time > 0.0,
            read.prev + 236
        )?;
        Some(motion.run_time)
    } else {
        assert_that!(
            "object motion run time",
            motion.run_time == 0.0,
            read.prev + 236
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

fn write_pg(
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
            return Err(assert_with_msg!(
                "No-altitude gravity is not supported in RC"
            ));
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
        flags |= ObjectMotionFlags::TRANSLATION_RANGE;

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
    let mut bounce_seq0_index = 0;
    if let Some(bounce) = &motion.bounce_sequence {
        if bounce.water.is_some() {
            return Err(assert_with_msg!(
                "Water bounce sequence is not supported in RC"
            ));
        }
        if bounce.lava.is_some() {
            return Err(assert_with_msg!(
                "Lava bounce sequence is not supported in RC"
            ));
        }

        flags |= ObjectMotionFlags::BOUNCE_SEQUENCE;
        if let Some(s) = &bounce.default {
            bounce_seq0_name = Ascii::from_str_padded(s);
            bounce_seq0_index = -1;
        }
    };

    let mut bounce_snd0_index = index!(0);
    let mut bounce_snd0_volume = 0.0;
    if let Some(bounce) = &motion.bounce_sound {
        if bounce.water.is_some() {
            return Err(assert_with_msg!(
                "Water bounce sound is not supported in RC"
            ));
        }
        if bounce.lava.is_some() {
            return Err(assert_with_msg!("Lava bounce sound is not supported in RC"));
        }

        flags |= ObjectMotionFlags::BOUNCE_SOUND;
        if let Some(s) = &bounce.default {
            bounce_snd0_index = anim_def.stc_sound_to_index(&s.name)?;
            bounce_snd0_volume = s.volume;
        }
    };

    if motion.run_time.is_some() {
        flags |= ObjectMotionFlags::RUN_TIME;
    }
    let run_time = motion.run_time.unwrap_or(0.0);

    let object_motion = ObjectMotionRcC {
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
        bounce_seq0_index,
        bounce_snd0_index,
        bounce_snd0_volume,
        run_time,
    };
    write.write_struct(&object_motion)?;
    Ok(())
}
