use super::EventRc;
use crate::types::{index, AnimDefLookup as _, Idx16, Idx32};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::{
    BounceSequence, BounceSound, ForwardRotation, ForwardRotationDistance, ForwardRotationTime,
    Gravity, GravityMode, ObjectMotion, ObjectMotionScale, ObjectMotionTranslation, XyzRotation,
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
        // const IMPACT_FORCE = 1 << 1;                // 0x00002
        const TRANSLATION = 1 << 2;                 // 0x00004
        const TRANSLATION_RANGE = 1 << 3;           // 0x00008
        // const TRANSLATION_MAX = 1 << 4;             // 0x00010
        const OBJECT_MOTION_UNK_4 = 1 << 4;             // 0x00010
        const XYZ_ROTATION = 1 << 5;                // 0x00020
        // const FORWARD_ROTATION_DISTANCE = 1 << 6;   // 0x00040
        const FORWARD_ROTATION_TIME = 1 << 7;       // 0x00080
        const SCALE = 1 << 8;                       // 0x00100
        const RUN_TIME = 1 << 10;                   // 0x00400
        const BOUNCE_SEQUENCE = 1 << 11;            // 0x00800
        const BOUNCE_SOUND = 1 << 12;               // 0x01000

        // const GRAVITY_COMPLEX = 1 << 13;            // 0x02000
        // const GRAVITY_NO_ALTITUDE = 1 << 14;        // 0x04000
    }
}

type Flags = Maybe<u32, ObjectMotionFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ObjectMotionRcC {
    flags: Flags,      // 000
    node_index: Idx16, // 004
    pad006: u16,       // 006
    unk008: f32,       // 008
    gravity: f32,      // 012
    morph: f32,        // 016

    trans_range_xz: Range,      // 020
    trans_range_y: Range,       // 028
    trans_range_delta: Range,   // 036
    trans_range_initial: Range, // 044

    trans_delta: Vec3,        // 052
    trans_initial: Vec3,      // 064
    trans_delta_copy: Vec3,   // 076
    trans_initial_copy: Vec3, // 088
    trans_range_rand: Vec3,   // 100

    fwd_rot_112: f32, // 112
    fwd_rot_116: f32, // 116
    unk120: f32,      // 120

    xyz_rot_124: f32, // 124
    xyz_rot_128: f32, // 128
    xyz_rot_132: f32, // 132
    xyz_rot_136: f32, // 136
    xyz_rot_140: f32, // 140
    xyz_rot_144: f32, // 144
    unk184: f32,      // 148
    unk188: f32,      // 152
    unk192: f32,      // 156

    scale_base: Vec3,    // 160
    scale_delta: Vec3,   // 172
    scale_current: Vec3, // 184

    bounce_sequence_name: Ascii<32>,        // 196
    bounce_sequence_index: i16,             // 228
    bounce_sound_static_sound_index: Idx16, // 230
    bounce_sound_full_volume_velocity: f32, // 232
    run_time: f32,                          // 236
}
impl_as_bytes!(ObjectMotionRcC, 240);

#[derive(Debug)]
struct XyzRot {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
}

const GRAVITY: f32 = -9.800000190734863;

impl EventRc for ObjectMotion {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(ObjectMotionRcC::SIZE)
    }

    #[inline]
    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!(
            "object motion size",
            size == ObjectMotionRcC::SIZE,
            read.offset
        )?;
        // TODO
        log::error!("ANIM DEF `{}`, `{}`", anim_def.name, anim_def.anim_name);
        let motion: ObjectMotionRcC = read.read_struct()?;

        let name = anim_def.node_from_index(motion.node_index, read.prev + 4)?;
        log::error!("OBJECT MOTION `{}`", name);

        let flags = assert_that!("object motion flags", flags motion.flags, read.prev + 0)?;
        // let name = anim_def.node_from_index(motion.node_index, read.prev + 4)?;
        assert_that!("object motion field 006", motion.pad006 == 0, read.prev + 6)?;
        assert_that!(
            "object motion field 008",
            motion.unk008 == 0.0,
            read.prev + 8
        )?;
        assert_that!(
            "object motion field 016",
            motion.morph == 0.0,
            read.prev + 16
        )?;

        let gravity = if flags.contains(ObjectMotionFlags::GRAVITY) {
            // DEFAULT or LOCAL
            // assert_that!(
            //     "object motion gravity",
            //     motion.gravity == GRAVITY,
            //     read.prev + 12
            // )?;
            Some(motion.gravity)
        } else {
            assert_that!(
                "object motion gravity",
                motion.gravity == 0.0,
                read.prev + 12
            )?;
            None
        };

        if flags.contains(ObjectMotionFlags::FORWARD_ROTATION_TIME) {
            assert_that!(
                "object motion field 120 (fwd rot)",
                motion.unk120 == 0.0,
                read.prev + 120
            )?;
        } else {
            assert_that!(
                "object motion field 112",
                motion.fwd_rot_112 == 0.0,
                read.prev + 112
            )?;
            assert_that!(
                "object motion field 116",
                motion.fwd_rot_116 == 0.0,
                read.prev + 116
            )?;
            assert_that!(
                "object motion field 120",
                motion.unk120 == 0.0,
                read.prev + 120
            )?;
        }

        // TODO
        if flags.contains(ObjectMotionFlags::XYZ_ROTATION) {
            let xyz_rot = XyzRot {
                a: motion.xyz_rot_124.to_degrees(),
                b: motion.xyz_rot_128.to_degrees(),
                c: motion.xyz_rot_132.to_degrees(),
                d: motion.xyz_rot_136.to_degrees(),
                e: motion.xyz_rot_140.to_degrees(),
                f: motion.xyz_rot_144.to_degrees(),
            };
            log::error!("{:#?}", xyz_rot);
        } else {
            assert_that!(
                "object motion xyz rot 124",
                motion.xyz_rot_124 == 0.0,
                read.prev + 124
            )?;
            assert_that!(
                "object motion xyz rot 128",
                motion.xyz_rot_128 == 0.0,
                read.prev + 128
            )?;
            assert_that!(
                "object motion xyz rot 132",
                motion.xyz_rot_132 == 0.0,
                read.prev + 132
            )?;
            assert_that!(
                "object motion xyz rot 136",
                motion.xyz_rot_136 == 0.0,
                read.prev + 136
            )?;
            assert_that!(
                "object motion xyz rot 140",
                motion.xyz_rot_140 == 0.0,
                read.prev + 140
            )?;
            assert_that!(
                "object motion xyz rot 144",
                motion.xyz_rot_144 == 0.0,
                read.prev + 144
            )?;
        }

        let bounce_sequence = if flags.contains(ObjectMotionFlags::BOUNCE_SEQUENCE) {
            let seq_name = assert_utf8(
                "object motion bounce sequence name",
                read.prev + 196,
                || motion.bounce_sequence_name.to_str_padded(),
            )?;
            assert_that!(
                "object motion bounce sequence index",
                motion.bounce_sequence_index == -1,
                read.prev + 228
            )?;
            Some(seq_name)
        } else {
            assert_that!("object motion bounce sequence name", zero motion.bounce_sequence_name, read.prev + 196)?;
            assert_that!(
                "object motion bounce sequence index",
                motion.bounce_sequence_index == 0,
                read.prev + 228
            )?;
            None
        };

        let bounce_sound = if flags.contains(ObjectMotionFlags::BOUNCE_SOUND) {
            let name = anim_def
                .stc_sound_from_index(motion.bounce_sound_static_sound_index, read.prev + 230)?;
            Some(BounceSound {
                name,
                volume: motion.bounce_sound_full_volume_velocity,
            })
        } else {
            assert_that!(
                "object motion bounce sound index",
                motion.bounce_sound_static_sound_index == index!(0),
                read.prev + 230
            )?;
            assert_that!(
                "object motion bounce sound volume",
                motion.bounce_sound_full_volume_velocity == 0.0,
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

        Ok(Self {
            node: "".to_string(),
            impact_force: false,
            gravity: None,
            translation_range: None,
            translation: None,
            forward_rotation: None,
            xyz_rotation: None,
            scale: None,
            bounce_sequence: None,
            bounce_sound,
            run_time,
        })
    }

    #[inline]
    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let buf = vec![0; 240];
        write.write_all(&buf)?;
        Ok(())
    }
}
