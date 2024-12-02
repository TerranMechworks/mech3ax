use super::{EventMw, EventPm};
use crate::types::{index, AnimDefLookup as _, Idx16};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::{
    BounceSequence, BounceSound, ForwardRotation, ForwardRotationDistance, ForwardRotationTime,
    Gravity, GravityMode, ObjectMotion, ObjectMotionScale, ObjectMotionTranslation,
    TranslationRange, XyzRotation,
};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::{Range, Vec3};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, assert_with_msg, Result};
use mech3ax_types::{bitflags, impl_as_bytes, AsBytes as _, Ascii, Maybe};
use std::io::{Read, Write};

bitflags! {
    struct ObjectMotionFlags: u32 {
        const GRAVITY = 1 << 0;                     // 0x00001
        const IMPACT_FORCE = 1 << 1;                // 0x00002
        const TRANSLATION = 1 << 2;                 // 0x00004
        const TRANSLATION_RANGE = 1 << 3;           // 0x00008
        const TRANSLATION_MAX = 1 << 4;             // 0x00010
        const XYZ_ROTATION = 1 << 5;                // 0x00020
        const FORWARD_ROTATION_DISTANCE = 1 << 6;   // 0x00040
        const FORWARD_ROTATION_TIME = 1 << 7;       // 0x00080
        const SCALE = 1 << 8;                       // 0x00100
        const RUN_TIME = 1 << 10;                   // 0x00400
        const BOUNCE_SEQ = 1 << 11;                 // 0x00800
        const BOUNCE_SOUND = 1 << 12;               // 0x01000
        const GRAVITY_COMPLEX = 1 << 13;            // 0x02000
        const GRAVITY_NO_ALTITUDE = 1 << 14;        // 0x04000
    }
}

type Flags = Maybe<u32, ObjectMotionFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ObjectMotionNgC {
    flags: Flags,               // 000
    node_index: Idx16,          // 004
    pad006: u16,                // 006
    zero008: f32,               // 008
    gravity: f32,               // 012
    morph: f32,                 // 016
    trans_range_xz: Range,      // 020
    trans_range_y: Range,       // 028
    trans_range_delta: Range,   // 036
    trans_range_initial: Range, // 044

    trans_delta: Vec3,        // 052
    trans_initial: Vec3,      // 064
    trans_delta_copy: Vec3,   // 076
    trans_initial_copy: Vec3, // 088
    unk100: Vec3,             // 100

    forward_rotation_1: f32, // 112
    forward_rotation_2: f32, // 116
    zero120: f32,            // 120

    xyz_rotation: Vec3,      // 124
    unk136: Vec3,            // 136
    xyz_rotation_copy: Vec3, // 148

    scale: Vec3,      // 160
    unk172: Vec3,     // 172
    scale_copy: Vec3, // 184

    bounce_seq0_name: Ascii<32>, // 196
    bounce_seq0_sentinel: i16,   // 228
    bounce_snd0_index: Idx16,    // 230
    bounce_snd0_volume: f32,     // 232
    bounce_seq1_name: Ascii<32>, // 236
    bounce_seq1_sentinel: i16,   // 268
    bounce_snd1_index: Idx16,    // 270
    bounce_snd1_volume: f32,     // 272
    bounce_seq2_name: Ascii<32>, // 276
    bounce_seq2_sentinel: i16,   // 308
    bounce_snd2_index: Idx16,    // 310
    bounce_snd2_volume: f32,     // 312

    run_time: f32, // 316
}
impl_as_bytes!(ObjectMotionNgC, 320);

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

    let flags = assert_that!("object motion flags", flags motion.flags, read.prev + 0)?;
    let node = anim_def.node_from_index(motion.node_index, read.prev + 4)?;
    assert_that!("object motion field 006", motion.pad006 == 0, read.prev + 6)?;

    assert_that!(
        "object motion field 008",
        motion.zero008 == 0.0,
        read.prev + 8
    )?;
    assert_that!(
        "object motion field 016",
        motion.morph == 0.0,
        read.prev + 16
    )?;

    let gravity_complex = flags.contains(ObjectMotionFlags::GRAVITY_COMPLEX);
    let gravity_no_alitude = flags.contains(ObjectMotionFlags::GRAVITY_NO_ALTITUDE);

    let gravity = if flags.contains(ObjectMotionFlags::GRAVITY) {
        let mode = if gravity_no_alitude {
            assert_that!(
                "object motion gravity complex",
                gravity_complex == false,
                read.prev + 0
            )?;
            GravityMode::NoAltitude
        } else if gravity_complex {
            GravityMode::Complex
        } else {
            GravityMode::Local
        };
        Some(Gravity {
            mode,
            value: motion.gravity,
        })
    } else {
        assert_that!(
            "object motion gravity complex",
            gravity_complex == false,
            read.prev + 0
        )?;
        assert_that!(
            "object motion gravity no altitude",
            gravity_no_alitude == false,
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
            delta: motion.trans_range_delta,
            initial: motion.trans_range_initial,
        })
    } else {
        assert_that!(
            "object motion trans range xz min",
            motion.trans_range_xz.min == 0.0,
            read.prev + 20
        )?;
        assert_that!(
            "object motion trans range xz max",
            motion.trans_range_xz.max == 0.0,
            read.prev + 24
        )?;
        assert_that!(
            "object motion trans range y min",
            motion.trans_range_y.min == 0.0,
            read.prev + 28
        )?;
        assert_that!(
            "object motion trans range y max",
            motion.trans_range_y.max == 0.0,
            read.prev + 32
        )?;
        assert_that!(
            "object motion trans range delta min",
            motion.trans_range_delta.min == 0.0,
            read.prev + 36
        )?;
        assert_that!(
            "object motion trans range delta max",
            motion.trans_range_delta.max == 0.0,
            read.prev + 40
        )?;
        assert_that!(
            "object motion trans range initial min",
            motion.trans_range_initial.min == 0.0,
            read.prev + 44
        )?;
        assert_that!(
            "object motion trans range initial max",
            motion.trans_range_initial.max == 0.0,
            read.prev + 48
        )?;
        None
    };

    let translation = if flags.contains(ObjectMotionFlags::TRANSLATION) {
        Some(ObjectMotionTranslation {
            delta: motion.trans_delta,
            initial: motion.trans_initial,
            unk: motion.unk100,
        })
    } else {
        assert_that!(
            "object motion trans delta",
            motion.trans_delta == Vec3::DEFAULT,
            read.prev + 52
        )?;
        assert_that!(
            "object motion trans initial",
            motion.trans_initial == Vec3::DEFAULT,
            read.prev + 64
        )?;
        assert_that!(
            "object motion field 100",
            motion.unk100 == Vec3::DEFAULT,
            read.prev + 100
        )?;
        None
    };
    assert_that!(
        "object motion trans delta copy",
        motion.trans_delta_copy == Vec3::DEFAULT,
        read.prev + 76
    )?;
    assert_that!(
        "object motion trans initial copy",
        motion.trans_initial_copy == Vec3::DEFAULT,
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
            v1: motion.forward_rotation_1,
            v2: motion.forward_rotation_2,
        }))
    } else if forward_rotation_dist {
        assert_that!(
            "object motion fwd rot 2",
            motion.forward_rotation_2 == 0.0,
            read.prev + 116
        )?;
        Some(ForwardRotation::Distance(ForwardRotationDistance {
            v1: motion.forward_rotation_1,
        }))
    } else {
        assert_that!(
            "object motion fwd rot 1",
            motion.forward_rotation_1 == 0.0,
            read.prev + 112
        )?;
        assert_that!(
            "object motion fwd rot 2",
            motion.forward_rotation_2 == 0.0,
            read.prev + 116
        )?;
        None
    };
    assert_that!(
        "object motion field 120",
        motion.zero120 == 0.0,
        read.prev + 120
    )?;

    let xyz_rotation = if flags.contains(ObjectMotionFlags::XYZ_ROTATION) {
        Some(XyzRotation {
            value: motion.xyz_rotation,
            unk: motion.unk136,
        })
    } else {
        assert_that!(
            "object motion xyz rot",
            motion.xyz_rotation == Vec3::DEFAULT,
            read.prev + 124
        )?;
        assert_that!(
            "object motion field 136",
            motion.unk136 == Vec3::DEFAULT,
            read.prev + 136
        )?;
        None
    };
    assert_that!(
        "object motion xyz rot copy",
        motion.xyz_rotation_copy == Vec3::DEFAULT,
        read.prev + 148
    )?;

    let scale = if flags.contains(ObjectMotionFlags::SCALE) {
        Some(ObjectMotionScale {
            value: motion.scale,
            unk: motion.unk172,
        })
    } else {
        assert_that!(
            "object motion scale",
            motion.scale == Vec3::DEFAULT,
            read.prev + 160
        )?;
        assert_that!(
            "object motion field 172",
            motion.unk172 == Vec3::DEFAULT,
            read.prev + 172
        )?;
        None
    };
    assert_that!(
        "object motion scale copy",
        motion.scale_copy == Vec3::DEFAULT,
        read.prev + 182
    )?;

    assert_that!(
        "object motion bounce seq 0 sentinel",
        motion.bounce_seq0_sentinel == -1,
        read.prev + 228
    )?;
    assert_that!(
        "object motion bounce seq 1 sentinel",
        motion.bounce_seq1_sentinel == -1,
        read.prev + 268
    )?;
    assert_that!(
        "object motion bounce seq 2 sentinel",
        motion.bounce_seq2_sentinel == -1,
        read.prev + 308
    )?;

    let bounce_sequence = if flags.contains(ObjectMotionFlags::BOUNCE_SEQ) {
        let seq_name0 = if !motion.bounce_seq0_name.first_is_zero() {
            let bounce_seq0 =
                assert_utf8("object motion bounce seq 0 name", read.prev + 196, || {
                    motion.bounce_seq0_name.to_str_padded()
                })?;
            Some(bounce_seq0)
        } else {
            return Err(assert_with_msg!(
                "Expected at least one bounce sequence (at {})",
                read.prev + 196
            ));
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

        Some(BounceSequence {
            seq_name0,
            seq_name1,
            seq_name2,
        })
    } else {
        assert_that!("object motion bounce seq 0", zero motion.bounce_seq0_name, read.prev + 196)?;
        assert_that!("object motion bounce seq 1", zero motion.bounce_seq1_name, read.prev + 236)?;
        assert_that!("object motion bounce seq 2", zero motion.bounce_seq2_name, read.prev + 276)?;
        None
    };

    let bounce_sound = if flags.contains(ObjectMotionFlags::BOUNCE_SOUND) {
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
        None
    };

    // these are never used, regardless of the flag
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
        gravity,
        impact_force: flags.contains(ObjectMotionFlags::IMPACT_FORCE),
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
    let gravity = if let Some(gravity_info) = &motion.gravity {
        flags |= ObjectMotionFlags::GRAVITY;
        match &gravity_info.mode {
            GravityMode::Complex => flags |= ObjectMotionFlags::GRAVITY_COMPLEX,
            GravityMode::NoAltitude => flags |= ObjectMotionFlags::GRAVITY_NO_ALTITUDE,
            GravityMode::Local => {}
        }
        gravity_info.value
    } else {
        0.0
    };

    if motion.impact_force {
        flags |= ObjectMotionFlags::IMPACT_FORCE;
    }
    if motion.translation_range.is_some() {
        flags |= ObjectMotionFlags::TRANSLATION_RANGE;
    }
    let TranslationRange {
        xz: trans_range_xz,
        y: trans_range_y,
        delta: trans_range_delta,
        initial: trans_range_initial,
    } = motion
        .translation_range
        .clone()
        .unwrap_or_else(|| TranslationRange {
            xz: Range::DEFAULT,
            y: Range::DEFAULT,
            delta: Range::DEFAULT,
            initial: Range::DEFAULT,
        });

    let (trans_delta, trans_initial, unk100) = if let Some(ObjectMotionTranslation {
        delta,
        initial,
        unk,
    }) = &motion.translation
    {
        flags |= ObjectMotionFlags::TRANSLATION;
        (*delta, *initial, *unk)
    } else {
        (Vec3::DEFAULT, Vec3::DEFAULT, Vec3::DEFAULT)
    };

    let (forward_rotation_1, forward_rotation_2) = match &motion.forward_rotation {
        Some(ForwardRotation::Time(ForwardRotationTime {
            v1: forward_rotation_1,
            v2: forward_rotation_2,
        })) => {
            flags |= ObjectMotionFlags::FORWARD_ROTATION_TIME;
            (*forward_rotation_1, *forward_rotation_2)
        }
        Some(ForwardRotation::Distance(ForwardRotationDistance {
            v1: forward_rotation_1,
        })) => {
            flags |= ObjectMotionFlags::FORWARD_ROTATION_DISTANCE;
            (*forward_rotation_1, 0.0)
        }
        None => (0.0, 0.0),
    };

    let (xyz_rotation, unk136) = if let Some(XyzRotation { value, unk }) = &motion.xyz_rotation {
        flags |= ObjectMotionFlags::XYZ_ROTATION;
        (*value, *unk)
    } else {
        (Vec3::DEFAULT, Vec3::DEFAULT)
    };

    let (scale, unk172) = if let Some(ObjectMotionScale { value, unk }) = &motion.scale {
        flags |= ObjectMotionFlags::SCALE;
        (*value, *unk)
    } else {
        (Vec3::DEFAULT, Vec3::DEFAULT)
    };

    let mut bounce_seq0_name = Ascii::zero();
    let mut bounce_seq1_name = Ascii::zero();
    let mut bounce_seq2_name = Ascii::zero();

    if let Some(bounce_seq) = &motion.bounce_sequence {
        flags |= ObjectMotionFlags::BOUNCE_SEQ;

        if let Some(name) = bounce_seq.seq_name0.as_ref() {
            bounce_seq0_name = Ascii::from_str_padded(name);
        }
        if let Some(name) = bounce_seq.seq_name1.as_ref() {
            bounce_seq1_name = Ascii::from_str_padded(name);
        }
        if let Some(name) = bounce_seq.seq_name2.as_ref() {
            bounce_seq2_name = Ascii::from_str_padded(name);
        }
    }

    let (bounce_snd0_index, bounce_snd0_volume) = if let Some(bounce_sound) = &motion.bounce_sound {
        flags |= ObjectMotionFlags::BOUNCE_SOUND;
        let sound_index = anim_def.stc_sound_to_index(&bounce_sound.name)?;
        (sound_index, bounce_sound.volume)
    } else {
        (index!(0), 0.0)
    };

    if motion.run_time.is_some() {
        flags |= ObjectMotionFlags::RUN_TIME;
    }

    let object_motion = ObjectMotionNgC {
        flags: flags.maybe(),
        node_index,
        pad006: 0,
        zero008: 0.0,
        gravity,
        morph: 0.0,
        trans_range_xz,
        trans_range_y,
        trans_range_delta,
        trans_range_initial,
        trans_delta,
        trans_initial,
        trans_delta_copy: Vec3::DEFAULT,
        trans_initial_copy: Vec3::DEFAULT,
        unk100,
        forward_rotation_1,
        forward_rotation_2,
        zero120: 0.0,
        xyz_rotation,
        unk136,
        xyz_rotation_copy: Vec3::DEFAULT,
        scale,
        unk172,
        scale_copy: Vec3::DEFAULT,
        bounce_seq0_name,
        bounce_seq0_sentinel: -1,
        bounce_snd0_index,
        bounce_snd0_volume,
        bounce_seq1_name,
        bounce_seq1_sentinel: -1,
        bounce_snd1_index: index!(0),
        bounce_snd1_volume: 0.0,
        bounce_seq2_name,
        bounce_seq2_sentinel: -1,
        bounce_snd2_index: index!(0),
        bounce_snd2_volume: 0.0,
        run_time: motion.run_time.unwrap_or(0.0),
    };
    write.write_struct(&object_motion)?;
    Ok(())
}
