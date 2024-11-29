use super::ScriptObject;
use crate::types::AnimDefLookup as _;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::{
    AtNode, Interval, IntervalType, PufferState, PufferStateCycleTextures,
};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::{Range, Vec3};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{bitflags, impl_as_bytes, AsBytes as _, Ascii, Bool32, Maybe, Zeros};
use std::io::{Read, Write};

bitflags! {
    struct PufferStateFlags: u32 {
        const TRANSLATE = 1 << 0;
        // this might not be right?
        const GROWTH_FACTOR = 1 << 1;
        // this might not be right?
        const STATE = 1 << 2;
        const LOCAL_VELOCITY = 1 << 3;
        const WORLD_VELOCITY = 1 << 4;
        const MIN_RANDOM_VELOCITY = 1 << 5;
        const MAX_RANDOM_VELOCITY = 1 << 6;
        const INTERVAL_TYPE = 1 << 7;
        const INTERVAL_VALUE = 1 << 8;
        const SIZE_RANGE = 1 << 9;
        const LIFETIME_RANGE = 1 << 10;
        const DEVIATION_DISTANCE = 1 << 11;
        const FADE_RANGE = 1 << 12;
        const ACTIVE = 1 << 13;
        const CYCLE_TEXTURE = 1 << 14;
        const START_AGE_RANGE = 1 << 15;
        const WORLD_ACCELERATION = 1 << 16;
        const FRICTION = 1 << 17;
        // there are more possible values (that are never set in the file)
    }
}

type Flags = Maybe<u32, PufferStateFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct PufferStateC {
    name: Ascii<32>,           // 000
    puffer_index: u32,         // 032
    flags: Flags,              // 036
    active_state: i32,         // 040
    node_index: u32,           // 044
    translation: Vec3,         // 048
    local_velocity: Vec3,      // 060
    world_velocity: Vec3,      // 072
    min_random_velocity: Vec3, // 084
    max_random_velocity: Vec3, // 096
    world_acceleration: Vec3,  // 108
    interval_type: Bool32,     // 120
    interval_value: f32,       // 124
    size_range: Range,         // 128
    lifetime_range: Range,     // 136
    start_age_range: Range,    // 144
    deviation_distance: f32,   // 152
    zero156: Range,            // 156
    fade_range: Range,         // 164
    friction: f32,             // 172
    zero176: u32,              // 176
    zero180: u32,              // 180
    zero184: u32,              // 184
    zero188: u32,              // 188
    tex192: Ascii<36>,         // 192
    tex228: Ascii<36>,         // 228
    tex264: Ascii<36>,         // 264
    tex300: Ascii<36>,         // 300
    tex336: Ascii<36>,         // 336
    tex372: Ascii<36>,         // 372
    zero408: Zeros<120>,       // 408
    unk528: u32,               // 528
    zero532: u32,              // 532
    unk536: f32,               // 536
    unk540: f32,               // 540
    growth_factor: f32,        // 544
    zero548: Zeros<32>,        // 548
}
impl_as_bytes!(PufferStateC, 580);

impl ScriptObject for PufferState {
    const INDEX: u8 = 42;
    const SIZE: u32 = PufferStateC::SIZE;

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("puffer state size", size == Self::SIZE, read.offset)?;
        let puffer_state: PufferStateC = read.read_struct()?;
        let name = assert_utf8("puffer state name", read.prev + 0, || {
            puffer_state.name.to_str_padded()
        })?;
        let expected_name =
            anim_def.puffer_from_index(puffer_state.puffer_index as usize, read.prev + 32)?;
        let name_ref = &name;
        assert_that!(
            "puffer state name",
            name_ref == &expected_name,
            read.prev + 0
        )?;
        let flags = assert_that!("puffer state flags", flags puffer_state.flags, read.prev + 36)?;

        let state = flags.contains(PufferStateFlags::STATE);
        if !state {
            // if the puffer state is disabled/inactive, then nothing else may be
            // specified. this ensures all further branches check for zero values.
            assert_that!(
                "puffer state flags",
                flags == PufferStateFlags::empty(),
                read.prev + 36
            )?;
        }

        let active_state = if flags.contains(PufferStateFlags::ACTIVE) {
            assert_that!("puffer state active", 1 <= puffer_state.active_state <= 5, read.prev + 40)?;
            Some(puffer_state.active_state)
        } else {
            assert_that!(
                "puffer state active",
                puffer_state.active_state == -1,
                read.prev + 40
            )?;
            None
        };

        // TODO: how does this flag interact?
        let translate = flags.contains(PufferStateFlags::TRANSLATE);
        let at_node = if puffer_state.node_index != 0 {
            let node =
                anim_def.node_from_index(puffer_state.node_index as usize, read.prev + 44)?;
            Some(AtNode {
                node,
                translation: puffer_state.translation,
            })
        } else {
            assert_that!(
                "puffer state translation",
                puffer_state.translation == Vec3::DEFAULT,
                read.prev + 48
            )?;
            None
        };

        let local_velocity = if flags.contains(PufferStateFlags::LOCAL_VELOCITY) {
            Some(puffer_state.local_velocity)
        } else {
            assert_that!(
                "puffer state local velocity",
                puffer_state.local_velocity == Vec3::DEFAULT,
                read.prev + 60
            )?;
            None
        };

        let world_velocity = if flags.contains(PufferStateFlags::WORLD_VELOCITY) {
            Some(puffer_state.world_velocity)
        } else {
            assert_that!(
                "puffer state world velocity",
                puffer_state.world_velocity == Vec3::DEFAULT,
                read.prev + 72
            )?;
            None
        };

        let min_random_velocity = if flags.contains(PufferStateFlags::MIN_RANDOM_VELOCITY) {
            Some(puffer_state.min_random_velocity)
        } else {
            assert_that!(
                "puffer state min rnd velocity",
                puffer_state.min_random_velocity == Vec3::DEFAULT,
                read.prev + 84
            )?;
            None
        };

        let max_random_velocity = if flags.contains(PufferStateFlags::MAX_RANDOM_VELOCITY) {
            Some(puffer_state.max_random_velocity)
        } else {
            assert_that!(
                "puffer state max rnd velocity",
                puffer_state.max_random_velocity == Vec3::DEFAULT,
                read.prev + 96
            )?;
            None
        };

        let world_acceleration = if flags.contains(PufferStateFlags::WORLD_ACCELERATION) {
            Some(puffer_state.world_acceleration)
        } else {
            assert_that!(
                "puffer state world accel",
                puffer_state.world_acceleration == Vec3::DEFAULT,
                read.prev + 108
            )?;
            None
        };

        let interval_type = if flags.contains(PufferStateFlags::INTERVAL_TYPE) {
            let interval_type = assert_that!("puffer state interval type", bool puffer_state.interval_type, read.prev + 120)?;
            if interval_type {
                IntervalType::Distance
            } else {
                IntervalType::Time
            }
        } else {
            assert_that!(
                "puffer state interval type",
                puffer_state.interval_type == Bool32::FALSE,
                read.prev + 120
            )?;
            IntervalType::Unset
        };
        assert_that!(
            "puffer state interval value",
            puffer_state.interval_value >= 0.0,
            read.prev + 124
        )?;

        // TODO: does not obey the flag?
        let interval = Interval {
            interval_type,
            interval_value: puffer_state.interval_value,
            flag: flags.contains(PufferStateFlags::INTERVAL_VALUE),
        };

        let size_range = if flags.contains(PufferStateFlags::SIZE_RANGE) {
            assert_that!(
                "puffer state size range min",
                puffer_state.size_range.min > 0.0,
                read.prev + 128
            )?;
            assert_that!(
                "puffer state size range max",
                puffer_state.size_range.max > puffer_state.size_range.min,
                read.prev + 132
            )?;
            Some(puffer_state.size_range)
        } else {
            assert_that!(
                "puffer state size range",
                puffer_state.size_range == Range::DEFAULT,
                read.prev + 128
            )?;
            None
        };

        let lifetime_range = if flags.contains(PufferStateFlags::LIFETIME_RANGE) {
            assert_that!(
                "puffer state lifetime range min",
                puffer_state.lifetime_range.min > 0.0,
                read.prev + 136
            )?;
            // TODO: this doesn't follow ordering?
            assert_that!(
                "puffer state lifetime range max",
                puffer_state.lifetime_range.max > 0.0,
                read.prev + 140
            )?;
            Some(puffer_state.lifetime_range)
        } else {
            assert_that!(
                "puffer state lifetime range",
                puffer_state.lifetime_range == Range::DEFAULT,
                read.prev + 136
            )?;
            None
        };

        let start_age_range = if flags.contains(PufferStateFlags::START_AGE_RANGE) {
            assert_that!(
                "puffer state start age range min",
                puffer_state.start_age_range.min >= 0.0,
                read.prev + 144
            )?;
            assert_that!(
                "puffer state start age range max",
                puffer_state.start_age_range.max > puffer_state.start_age_range.min,
                read.prev + 148
            )?;
            Some(puffer_state.start_age_range)
        } else {
            assert_that!(
                "puffer state start age range",
                puffer_state.start_age_range == Range::DEFAULT,
                read.prev + 144
            )?;
            None
        };

        let deviation_distance = if flags.contains(PufferStateFlags::DEVIATION_DISTANCE) {
            assert_that!(
                "puffer state deviation distance",
                puffer_state.deviation_distance > 0.0,
                read.prev + 152
            )?;
            Some(puffer_state.deviation_distance)
        } else {
            assert_that!(
                "puffer state deviation distance",
                puffer_state.deviation_distance == 0.0,
                read.prev + 152
            )?;
            None
        };

        assert_that!(
            "puffer state field 156",
            puffer_state.zero156 == Range::DEFAULT,
            read.prev + 156
        )?;

        let fade_range = if flags.contains(PufferStateFlags::FADE_RANGE) {
            assert_that!(
                "puffer state fade range near",
                puffer_state.fade_range.min > 0.0,
                read.prev + 164
            )?;
            assert_that!(
                "puffer state fade range max",
                puffer_state.fade_range.max > puffer_state.fade_range.min,
                read.prev + 168
            )?;
            Some(puffer_state.fade_range)
        } else {
            assert_that!(
                "puffer state fade range",
                puffer_state.fade_range == Range::DEFAULT,
                read.prev + 164
            )?;
            None
        };

        let friction = if flags.contains(PufferStateFlags::FRICTION) {
            assert_that!(
                "puffer state friction",
                puffer_state.friction >= 0.0,
                read.prev + 172
            )?;
            Some(puffer_state.friction)
        } else {
            assert_that!(
                "puffer state friction",
                puffer_state.friction == 0.0,
                read.prev + 172
            )?;
            None
        };

        assert_that!(
            "puffer state field 176",
            puffer_state.zero176 == 0,
            read.prev + 176
        )?;
        assert_that!(
            "puffer state field 180",
            puffer_state.zero180 == 0,
            read.prev + 180
        )?;
        assert_that!(
            "puffer state field 184",
            puffer_state.zero184 == 0,
            read.prev + 184
        )?;
        assert_that!(
            "puffer state field 188",
            puffer_state.zero188 == 0,
            read.prev + 188
        )?;

        let textures = if flags.contains(PufferStateFlags::CYCLE_TEXTURE) {
            let texture1 = if !puffer_state.tex192.first_is_zero() {
                Some(assert_utf8(
                    "puffer state texture 192",
                    read.prev + 192,
                    || puffer_state.tex192.to_str_padded(),
                )?)
            } else {
                assert_that!(
                    "puffer state texture 192",
                    zero puffer_state.tex192,
                    read.prev + 192
                )?;
                None
            };
            let texture2 = if !puffer_state.tex228.first_is_zero() {
                Some(assert_utf8(
                    "puffer state texture 228",
                    read.prev + 228,
                    || puffer_state.tex228.to_str_padded(),
                )?)
            } else {
                assert_that!(
                    "puffer state texture 228",
                    zero puffer_state.tex228,
                    read.prev + 228
                )?;
                None
            };
            let texture3 = if !puffer_state.tex264.first_is_zero() {
                Some(assert_utf8(
                    "puffer state texture 264",
                    read.prev + 264,
                    || puffer_state.tex264.to_str_padded(),
                )?)
            } else {
                assert_that!(
                    "puffer state texture 264",
                    zero puffer_state.tex264,
                    read.prev + 264
                )?;
                None
            };
            let texture4 = if !puffer_state.tex300.first_is_zero() {
                Some(assert_utf8(
                    "puffer state texture 300",
                    read.prev + 300,
                    || puffer_state.tex300.to_str_padded(),
                )?)
            } else {
                assert_that!(
                    "puffer state texture 300",
                    zero puffer_state.tex300,
                    read.prev + 300
                )?;
                None
            };
            let texture5 = if !puffer_state.tex336.first_is_zero() {
                Some(assert_utf8(
                    "puffer state texture 336",
                    read.prev + 336,
                    || puffer_state.tex336.to_str_padded(),
                )?)
            } else {
                assert_that!(
                    "puffer state texture 336",
                    zero puffer_state.tex336,
                    read.prev + 336
                )?;
                None
            };
            let texture6 = if !puffer_state.tex372.first_is_zero() {
                Some(assert_utf8(
                    "puffer state texture 372",
                    read.prev + 372,
                    || puffer_state.tex372.to_str_padded(),
                )?)
            } else {
                assert_that!(
                    "puffer state texture 372",
                    zero puffer_state.tex372,
                    read.prev + 372
                )?;
                None
            };
            Some(PufferStateCycleTextures {
                texture1,
                texture2,
                texture3,
                texture4,
                texture5,
                texture6,
            })
        } else {
            assert_that!(
                "puffer state texture 192",
                zero puffer_state.tex192,
                read.prev + 192
            )?;
            assert_that!(
                "puffer state texture 228",
                zero puffer_state.tex228,
                read.prev + 228
            )?;
            assert_that!(
                "puffer state texture 264",
                zero puffer_state.tex264,
                read.prev + 264
            )?;
            assert_that!(
                "puffer state texture 300",
                zero puffer_state.tex300,
                read.prev + 300
            )?;
            assert_that!(
                "puffer state texture 336",
                zero puffer_state.tex336,
                read.prev + 336
            )?;
            assert_that!(
                "puffer state texture 372",
                zero puffer_state.tex372,
                read.prev + 372
            )?;
            None
        };

        assert_that!(
            "puffer state field 408",
            zero puffer_state.zero408,
            read.prev + 408
        )?;
        assert_that!(
            "puffer state field 532",
            puffer_state.zero532 == 0,
            read.prev + 532
        )?;
        if active_state.is_some() {
            assert_that!(
                "puffer state field 528",
                puffer_state.unk528 == 2,
                read.prev + 528
            )?;
            assert_that!(
                "puffer state field 536",
                puffer_state.unk536 == 1.0,
                read.prev + 536
            )?;
            assert_that!(
                "puffer state field 540",
                puffer_state.unk540 == 1.0,
                read.prev + 540
            )?;
        } else {
            assert_that!(
                "puffer state field 528",
                puffer_state.unk528 == 0,
                read.prev + 528
            )?;
            assert_that!(
                "puffer state field 536",
                puffer_state.unk536 == 0.0,
                read.prev + 536
            )?;
            assert_that!(
                "puffer state field 540",
                puffer_state.unk540 == 0.0,
                read.prev + 540
            )?;
        }
        assert_that!(
            "puffer state field 548",
            zero puffer_state.zero548,
            read.prev + 548
        )?;

        let growth_factor = if flags.contains(PufferStateFlags::GROWTH_FACTOR) {
            assert_that!(
                "puffer state growth factor",
                puffer_state.growth_factor > 0.0,
                read.prev + 544
            )?;
            Some(puffer_state.growth_factor)
        } else {
            assert_that!(
                "puffer state growth factor",
                puffer_state.growth_factor == 0.0,
                read.prev + 544
            )?;
            None
        };
        Ok(Self {
            name,
            state,
            translate,
            active_state,
            at_node,
            local_velocity,
            world_velocity,
            min_random_velocity,
            max_random_velocity,
            world_acceleration,
            interval,
            size_range,
            lifetime_range,
            start_age_range,
            deviation_distance,
            fade_range,
            friction,
            textures,
            growth_factor,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let name = Ascii::from_str_padded(&self.name);
        let puffer_index = anim_def.puffer_to_index(&self.name)? as u32;
        let mut flags = PufferStateFlags::empty();
        if self.active_state.is_some() {
            flags |= PufferStateFlags::ACTIVE;
        }
        if self.state {
            flags |= PufferStateFlags::STATE;
        }
        if self.translate {
            flags |= PufferStateFlags::TRANSLATE;
        }
        let (node_index, translation) = if let Some(at_node) = &self.at_node {
            let node_index = anim_def.node_to_index(&at_node.node)? as u32;
            (node_index, at_node.translation)
        } else {
            (0, Vec3::DEFAULT)
        };
        if self.local_velocity.is_some() {
            flags |= PufferStateFlags::LOCAL_VELOCITY;
        }
        if self.world_velocity.is_some() {
            flags |= PufferStateFlags::WORLD_VELOCITY;
        }
        if self.min_random_velocity.is_some() {
            flags |= PufferStateFlags::MIN_RANDOM_VELOCITY;
        }
        if self.max_random_velocity.is_some() {
            flags |= PufferStateFlags::MAX_RANDOM_VELOCITY;
        }
        if self.world_acceleration.is_some() {
            flags |= PufferStateFlags::WORLD_ACCELERATION;
        }
        let interval_type = match self.interval.interval_type {
            IntervalType::Unset => false,
            IntervalType::Distance => {
                flags |= PufferStateFlags::INTERVAL_TYPE;
                true
            }
            IntervalType::Time => {
                flags |= PufferStateFlags::INTERVAL_TYPE;
                false
            }
        };
        if self.interval.flag {
            flags |= PufferStateFlags::INTERVAL_VALUE;
        }
        if self.size_range.is_some() {
            flags |= PufferStateFlags::SIZE_RANGE;
        }
        if self.lifetime_range.is_some() {
            flags |= PufferStateFlags::LIFETIME_RANGE;
        }
        if self.start_age_range.is_some() {
            flags |= PufferStateFlags::START_AGE_RANGE;
        }
        if self.deviation_distance.is_some() {
            flags |= PufferStateFlags::DEVIATION_DISTANCE;
        }
        if self.fade_range.is_some() {
            flags |= PufferStateFlags::FADE_RANGE;
        }
        if self.friction.is_some() {
            flags |= PufferStateFlags::FRICTION;
        }
        if self.growth_factor.is_some() {
            flags |= PufferStateFlags::GROWTH_FACTOR;
        }
        let mut tex192 = Ascii::zero();
        let mut tex228 = Ascii::zero();
        let mut tex264 = Ascii::zero();
        let mut tex300 = Ascii::zero();
        let mut tex336 = Ascii::zero();
        let mut tex372 = Ascii::zero();
        if let Some(textures) = &self.textures {
            flags |= PufferStateFlags::CYCLE_TEXTURE;
            if let Some(tex) = &textures.texture1 {
                tex192 = Ascii::from_str_padded(tex);
            }
            if let Some(tex) = &textures.texture2 {
                tex228 = Ascii::from_str_padded(tex);
            }
            if let Some(tex) = &textures.texture3 {
                tex264 = Ascii::from_str_padded(tex);
            }
            if let Some(tex) = &textures.texture4 {
                tex300 = Ascii::from_str_padded(tex);
            }
            if let Some(tex) = &textures.texture5 {
                tex336 = Ascii::from_str_padded(tex);
            }
            if let Some(tex) = &textures.texture6 {
                tex372 = Ascii::from_str_padded(tex);
            }
        }
        write.write_struct(&PufferStateC {
            name,
            puffer_index,
            flags: flags.maybe(),
            active_state: self.active_state.unwrap_or(-1),
            node_index,
            translation,
            local_velocity: self.local_velocity.unwrap_or(Vec3::DEFAULT),
            world_velocity: self.world_velocity.unwrap_or(Vec3::DEFAULT),
            min_random_velocity: self.min_random_velocity.unwrap_or(Vec3::DEFAULT),
            max_random_velocity: self.max_random_velocity.unwrap_or(Vec3::DEFAULT),
            world_acceleration: self.world_acceleration.unwrap_or(Vec3::DEFAULT),
            interval_type: interval_type.into(),
            interval_value: self.interval.interval_value,
            size_range: self.size_range.unwrap_or(Range::DEFAULT),
            lifetime_range: self.lifetime_range.unwrap_or(Range::DEFAULT),
            start_age_range: self.start_age_range.unwrap_or(Range::DEFAULT),
            deviation_distance: self.deviation_distance.unwrap_or(0.0),
            zero156: Range::DEFAULT,
            fade_range: self.fade_range.unwrap_or(Range::DEFAULT),
            friction: self.friction.unwrap_or(0.0),
            zero176: 0,
            zero180: 0,
            zero184: 0,
            zero188: 0,
            tex192,
            tex228,
            tex264,
            tex300,
            tex336,
            tex372,
            zero408: Zeros::new(),
            unk528: if self.active_state.is_some() { 2 } else { 0 },
            zero532: 0,
            unk536: if self.active_state.is_some() {
                1.0
            } else {
                0.0
            },
            unk540: if self.active_state.is_some() {
                1.0
            } else {
                0.0
            },
            growth_factor: self.growth_factor.unwrap_or(0.0),
            zero548: Zeros::new(),
        })?;
        Ok(())
    }
}
