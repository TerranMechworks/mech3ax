use super::ScriptObject;
use crate::types::AnimDefLookup as _;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::{
    BounceSequence, BounceSound, ForwardRotation, ForwardRotationDistance, ForwardRotationTime,
    Gravity, GravityMode, ObjectMotion, ObjectMotionScale, ObjectMotionTranslation, XyzRotation,
};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::{static_assert_size, Quaternion, ReprSize as _, Vec3};
use mech3ax_common::assert::{assert_all_zero, assert_utf8};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::string::{str_from_c_padded, str_to_c_padded};
use mech3ax_common::{assert_that, assert_with_msg, Result};
use mech3ax_debug::Ascii;
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ObjectMotionC {
    flags: u32,      // 000
    node_index: u32, // 004
    // GRAVITY
    zero008: f32, // 008
    gravity: f32, // 012
    zero016: f32, // 016
    // TRANSLATION RANGE MIN/MAX
    trans_range_min_1: f32, // 020
    trans_range_max_1: f32, // 024
    trans_range_min_2: f32, // 028
    trans_range_max_2: f32, // 032
    trans_range_min_3: f32, // 036
    trans_range_max_3: f32, // 040
    trans_range_min_4: f32, // 044
    trans_range_max_4: f32, // 048
    // TRANSLATION
    trans_delta: Vec3,   // 052
    trans_initial: Vec3, // 064
    // used for translation calculations
    trans_delta_copy: Vec3,   // 076
    trans_initial_copy: Vec3, // 088
    unk100: Vec3,             // 100
    // FORWARD_ROTATION
    forward_rotation_1: f32, // 112
    forward_rotation_2: f32, // 116
    zero120: f32,            // 120
    // XYZ_ROTATION
    xyz_rotation: Vec3, // 124
    unk136: Vec3,       // 136
    // used for xyz rotation calculations
    xyz_rotation_copy: Vec3, // 148
    // SCALE
    scale: Vec3,  // 160
    unk172: Vec3, // 172
    // used for scale calculations
    scale_copy: Vec3, // 184
    // BOUNCE SEQUENCE/SOUND
    bounce_seq0_name: Ascii<32>, // 196
    bounce_seq0_sentinel: i16,   // 228
    bounce_snd0_index: u16,      // 230
    bounce_snd0_volume: f32,     // 232
    bounce_seq1_name: Ascii<32>, // 236
    bounce_seq1_sentinel: i16,   // 268
    bounce_snd1_index: u16,      // 270
    bounce_snd1_volume: f32,     // 272
    bounce_seq2_name: Ascii<32>, // 276
    bounce_seq2_sentinel: i16,   // 308
    bounce_snd2_index: u16,      // 310
    bounce_snd2_volume: f32,     // 312
    // RUNTIME
    runtime: f32, // 316
}
static_assert_size!(ObjectMotionC, 320);

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct ObjectMotionFlags: u32 {
        const GRAVITY = 1 << 0; // 1
        const IMPACT_FORCE = 1 << 1; // 2
        const TRANSLATION = 1 << 2; // 4
        const TRANSLATION_MIN = 1 << 3; // 8
        const TRANSLATION_MAX = 1 << 4; // 16
        const XYZ_ROTATION = 1 << 5; // 32
        const FORWARD_ROTATION_DISTANCE = 1 << 6; // 64
        const FORWARD_ROTATION_TIME = 1 << 7; // 128
        const SCALE = 1 << 8; // 256
        const RUNTIME = 1 << 10; // 512
        const BOUNCE_SEQ = 1 << 11; // 2048
        const BOUNCE_SOUND = 1 << 12; // 4096
        const GRAVITY_COMPLEX = 1 << 13; // 8192
        const GRAVITY_NO_ALTITUDE = 1 << 14; // 16384
    }
}

impl ScriptObject for ObjectMotion {
    const INDEX: u8 = 10;
    const SIZE: u32 = ObjectMotionC::SIZE;

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("object motion size", size == Self::SIZE, read.offset)?;
        let object_motion: ObjectMotionC = read.read_struct()?;

        let flags = ObjectMotionFlags::from_bits(object_motion.flags).ok_or_else(|| {
            assert_with_msg!(
                "Expected valid object motion flags, but was 0x{:08X} (at {})",
                object_motion.flags,
                read.prev + 0
            )
        })?;
        let node = anim_def.node_from_index(object_motion.node_index as usize, read.prev + 4)?;

        assert_that!(
            "object motion field 008",
            object_motion.zero008 == 0.0,
            read.prev + 8
        )?;
        assert_that!(
            "object motion field 016",
            object_motion.zero016 == 0.0,
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
                value: object_motion.gravity,
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
                object_motion.gravity == 0.0,
                read.prev + 12
            )?;
            None
        };

        let translation_range_min = if flags.contains(ObjectMotionFlags::TRANSLATION_MIN) {
            Some(Quaternion {
                x: object_motion.trans_range_min_1,
                y: object_motion.trans_range_min_2,
                z: object_motion.trans_range_min_3,
                w: object_motion.trans_range_min_4,
            })
        } else {
            assert_that!(
                "object motion trans range min 1",
                object_motion.trans_range_min_1 == 0.0,
                read.prev + 20
            )?;
            assert_that!(
                "object motion trans range min 2",
                object_motion.trans_range_min_2 == 0.0,
                read.prev + 28
            )?;
            assert_that!(
                "object motion trans range min 3",
                object_motion.trans_range_min_3 == 0.0,
                read.prev + 36
            )?;
            assert_that!(
                "object motion trans range min 4",
                object_motion.trans_range_min_4 == 0.0,
                read.prev + 44
            )?;
            None
        };

        let translation_range_max = if flags.contains(ObjectMotionFlags::TRANSLATION_MAX) {
            Some(Quaternion {
                x: object_motion.trans_range_max_1,
                y: object_motion.trans_range_max_2,
                z: object_motion.trans_range_max_3,
                w: object_motion.trans_range_max_4,
            })
        } else {
            assert_that!(
                "object motion trans range max 1",
                object_motion.trans_range_max_1 == 0.0,
                read.prev + 24
            )?;
            assert_that!(
                "object motion trans range max 2",
                object_motion.trans_range_max_2 == 0.0,
                read.prev + 32
            )?;
            assert_that!(
                "object motion trans range max 3",
                object_motion.trans_range_max_3 == 0.0,
                read.prev + 40
            )?;
            assert_that!(
                "object motion trans range max 4",
                object_motion.trans_range_max_4 == 0.0,
                read.prev + 48
            )?;
            None
        };

        let translation = if flags.contains(ObjectMotionFlags::TRANSLATION) {
            Some(ObjectMotionTranslation {
                delta: object_motion.trans_delta,
                initial: object_motion.trans_initial,
                unk: object_motion.unk100,
            })
        } else {
            assert_that!(
                "object motion trans delta",
                object_motion.trans_delta == Vec3::DEFAULT,
                read.prev + 52
            )?;
            assert_that!(
                "object motion trans initial",
                object_motion.trans_initial == Vec3::DEFAULT,
                read.prev + 64
            )?;
            assert_that!(
                "object motion field 100",
                object_motion.unk100 == Vec3::DEFAULT,
                read.prev + 100
            )?;
            None
        };
        assert_that!(
            "object motion trans delta copy",
            object_motion.trans_delta_copy == Vec3::DEFAULT,
            read.prev + 76
        )?;
        assert_that!(
            "object motion trans initial copy",
            object_motion.trans_initial_copy == Vec3::DEFAULT,
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
                v1: object_motion.forward_rotation_1,
                v2: object_motion.forward_rotation_2,
            }))
        } else if forward_rotation_dist {
            assert_that!(
                "object motion fwd rot 2",
                object_motion.forward_rotation_2 == 0.0,
                read.prev + 116
            )?;
            Some(ForwardRotation::Distance(ForwardRotationDistance {
                v1: object_motion.forward_rotation_1,
            }))
        } else {
            assert_that!(
                "object motion fwd rot 1",
                object_motion.forward_rotation_1 == 0.0,
                read.prev + 112
            )?;
            assert_that!(
                "object motion fwd rot 2",
                object_motion.forward_rotation_2 == 0.0,
                read.prev + 116
            )?;
            None
        };
        assert_that!(
            "object motion field 120",
            object_motion.zero120 == 0.0,
            read.prev + 120
        )?;

        let xyz_rotation = if flags.contains(ObjectMotionFlags::XYZ_ROTATION) {
            Some(XyzRotation {
                value: object_motion.xyz_rotation,
                unk: object_motion.unk136,
            })
        } else {
            assert_that!(
                "object motion xyz rot",
                object_motion.xyz_rotation == Vec3::DEFAULT,
                read.prev + 124
            )?;
            assert_that!(
                "object motion field 136",
                object_motion.unk136 == Vec3::DEFAULT,
                read.prev + 136
            )?;
            None
        };
        assert_that!(
            "object motion xyz rot copy",
            object_motion.xyz_rotation_copy == Vec3::DEFAULT,
            read.prev + 148
        )?;

        let scale = if flags.contains(ObjectMotionFlags::SCALE) {
            Some(ObjectMotionScale {
                value: object_motion.scale,
                unk: object_motion.unk172,
            })
        } else {
            assert_that!(
                "object motion scale",
                object_motion.scale == Vec3::DEFAULT,
                read.prev + 160
            )?;
            assert_that!(
                "object motion field 172",
                object_motion.unk172 == Vec3::DEFAULT,
                read.prev + 172
            )?;
            None
        };
        assert_that!(
            "object motion scale copy",
            object_motion.scale_copy == Vec3::DEFAULT,
            read.prev + 182
        )?;

        assert_that!(
            "object motion bounce seq 0 sentinel",
            object_motion.bounce_seq0_sentinel == -1,
            read.prev + 228
        )?;
        assert_that!(
            "object motion bounce seq 1 sentinel",
            object_motion.bounce_seq1_sentinel == -1,
            read.prev + 268
        )?;
        assert_that!(
            "object motion bounce seq 2 sentinel",
            object_motion.bounce_seq2_sentinel == -1,
            read.prev + 308
        )?;

        let bounce_sequence = if flags.contains(ObjectMotionFlags::BOUNCE_SEQ) {
            let seq_name0 = if object_motion.bounce_seq0_name.0[0] != 0 {
                let bounce_seq0 =
                    assert_utf8("object motion bounce seq 0 name", read.prev + 196, || {
                        str_from_c_padded(&object_motion.bounce_seq0_name)
                    })?;
                Some(bounce_seq0)
            } else {
                return Err(assert_with_msg!(
                    "Expected at least one bounce sequence (at {})",
                    read.prev + 196
                ));
            };

            let seq_name1 = if object_motion.bounce_seq1_name.0[0] != 0 {
                let bounce_seq1 =
                    assert_utf8("object motion bounce seq 1 name", read.prev + 236, || {
                        str_from_c_padded(&object_motion.bounce_seq1_name)
                    })?;
                Some(bounce_seq1)
            } else {
                None
            };

            let seq_name2 = if object_motion.bounce_seq2_name.0[0] != 0 {
                let bounce_seq2 =
                    assert_utf8("object motion bounce seq 2 name", read.prev + 276, || {
                        str_from_c_padded(&object_motion.bounce_seq2_name)
                    })?;
                Some(bounce_seq2)
            } else {
                None
            };

            Some(BounceSequence {
                seq_name0,
                seq_name1,
                seq_name2,
            })
        } else {
            assert_all_zero(
                "object motion bounce seq 0",
                read.prev + 196,
                &object_motion.bounce_seq0_name.0,
            )?;
            assert_all_zero(
                "object motion bounce seq 1",
                read.prev + 236,
                &object_motion.bounce_seq1_name.0,
            )?;
            assert_all_zero(
                "object motion bounce seq 2",
                read.prev + 276,
                &object_motion.bounce_seq2_name.0,
            )?;
            None
        };

        let bounce_sound = if flags.contains(ObjectMotionFlags::BOUNCE_SOUND) {
            assert_that!(
                "object motion bounce snd 0 vol",
                object_motion.bounce_snd0_volume > 0.0,
                read.prev + 232
            )?;
            let sound_name = anim_def
                .sound_from_index(object_motion.bounce_snd0_index as usize, read.prev + 230)?;
            Some(BounceSound {
                name: sound_name,
                volume: object_motion.bounce_snd0_volume,
            })
        } else {
            assert_that!(
                "object motion bounce snd 0 index",
                object_motion.bounce_snd0_index == 0,
                read.prev + 230
            )?;
            assert_that!(
                "object motion bounce snd 0 vol",
                object_motion.bounce_snd0_volume == 0.0,
                read.prev + 232
            )?;
            None
        };

        // these are never used, regardless of the flag
        assert_that!(
            "object motion bounce snd 1 index",
            object_motion.bounce_snd1_index == 0,
            read.prev + 270
        )?;
        assert_that!(
            "object motion bounce snd 1 vol",
            object_motion.bounce_snd1_volume == 0.0,
            read.prev + 272
        )?;
        assert_that!(
            "object motion bounce snd 2 index",
            object_motion.bounce_snd2_index == 0,
            read.prev + 310
        )?;
        assert_that!(
            "object motion bounce snd 2 vol",
            object_motion.bounce_snd2_volume == 0.0,
            read.prev + 312
        )?;

        let runtime = if flags.contains(ObjectMotionFlags::RUNTIME) {
            assert_that!(
                "object motion runtime",
                object_motion.runtime > 0.0,
                read.prev + 316
            )?;
            Some(object_motion.runtime)
        } else {
            assert_that!(
                "object motion runtime",
                object_motion.runtime == 0.0,
                read.prev + 316
            )?;
            None
        };

        Ok(Self {
            node,
            gravity,
            impact_force: flags.contains(ObjectMotionFlags::IMPACT_FORCE),
            translation_range_min,
            translation_range_max,
            translation,
            forward_rotation,
            xyz_rotation,
            scale,
            bounce_sequence,
            bounce_sound,
            runtime,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let node_index = anim_def.node_to_index(&self.node)? as u32;
        let mut flags = ObjectMotionFlags::empty();
        let gravity = if let Some(gravity_info) = &self.gravity {
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

        if self.impact_force {
            flags |= ObjectMotionFlags::IMPACT_FORCE;
        }
        if self.translation_range_min.is_some() {
            flags |= ObjectMotionFlags::TRANSLATION_MIN;
        }
        let translation_range_min = self.translation_range_min.unwrap_or(Quaternion::DEFAULT);

        if self.translation_range_max.is_some() {
            flags |= ObjectMotionFlags::TRANSLATION_MAX;
        }
        let translation_range_max = self.translation_range_max.unwrap_or(Quaternion::DEFAULT);

        let (trans_delta, trans_initial, unk100) = if let Some(ObjectMotionTranslation {
            delta,
            initial,
            unk,
        }) = &self.translation
        {
            flags |= ObjectMotionFlags::TRANSLATION;
            (*delta, *initial, *unk)
        } else {
            (Vec3::DEFAULT, Vec3::DEFAULT, Vec3::DEFAULT)
        };

        let (forward_rotation_1, forward_rotation_2) = match &self.forward_rotation {
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

        let (xyz_rotation, unk136) = if let Some(XyzRotation { value, unk }) = &self.xyz_rotation {
            flags |= ObjectMotionFlags::XYZ_ROTATION;
            (*value, *unk)
        } else {
            (Vec3::DEFAULT, Vec3::DEFAULT)
        };

        let (scale, unk172) = if let Some(ObjectMotionScale { value, unk }) = &self.scale {
            flags |= ObjectMotionFlags::SCALE;
            (*value, *unk)
        } else {
            (Vec3::DEFAULT, Vec3::DEFAULT)
        };

        let mut bounce_seq0_name = Ascii::zero();
        let mut bounce_seq1_name = Ascii::zero();
        let mut bounce_seq2_name = Ascii::zero();

        if let Some(bounce_seq) = &self.bounce_sequence {
            flags |= ObjectMotionFlags::BOUNCE_SEQ;

            if let Some(name) = bounce_seq.seq_name0.as_ref() {
                str_to_c_padded(name, &mut bounce_seq0_name);
            }
            if let Some(name) = bounce_seq.seq_name1.as_ref() {
                str_to_c_padded(name, &mut bounce_seq1_name);
            }
            if let Some(name) = bounce_seq.seq_name2.as_ref() {
                str_to_c_padded(name, &mut bounce_seq2_name);
            }
        }

        let (bounce_snd0_index, bounce_snd0_volume) = if let Some(bounce_sound) = &self.bounce_sound
        {
            flags |= ObjectMotionFlags::BOUNCE_SOUND;
            let sound_index = anim_def.sound_to_index(&bounce_sound.name)? as u16;
            (sound_index, bounce_sound.volume)
        } else {
            (0, 0.0)
        };

        if self.runtime.is_some() {
            flags |= ObjectMotionFlags::RUNTIME;
        }

        write.write_struct(&ObjectMotionC {
            flags: flags.bits(),
            node_index,
            zero008: 0.0,
            gravity,
            zero016: 0.0,
            trans_range_min_1: translation_range_min.x,
            trans_range_max_1: translation_range_max.x,
            trans_range_min_2: translation_range_min.y,
            trans_range_max_2: translation_range_max.y,
            trans_range_min_3: translation_range_min.z,
            trans_range_max_3: translation_range_max.z,
            trans_range_min_4: translation_range_min.w,
            trans_range_max_4: translation_range_max.w,
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
            bounce_snd1_index: 0,
            bounce_snd1_volume: 0.0,
            bounce_seq2_name,
            bounce_seq2_sentinel: -1,
            bounce_snd2_index: 0,
            bounce_snd2_volume: 0.0,
            runtime: self.runtime.unwrap_or(0.0),
        })?;
        Ok(())
    }
}
