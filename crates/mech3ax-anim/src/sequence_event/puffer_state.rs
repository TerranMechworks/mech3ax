use super::types::AtNode;
use super::ScriptObject;
use crate::AnimDef;
use mech3ax_api_types::{static_assert_size, ReprSize as _, Vec2, Vec3};
use mech3ax_common::assert::{assert_all_zero, assert_utf8, AssertionError};
use mech3ax_common::io_ext::{CountingReader, WriteHelper};
use mech3ax_common::string::{str_from_c_padded, str_to_c_padded};
use mech3ax_common::{assert_that, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[repr(C)]
struct PufferStateC {
    name: [u8; 32],            // 000
    puffer_index: u32,         // 032
    flags: u32,                // 036
    active_state: i32,         // 040
    node_index: u32,           // 044
    translation: Vec3,         // 048
    local_velocity: Vec3,      // 060
    world_velocity: Vec3,      // 072
    min_random_velocity: Vec3, // 084
    max_random_velocity: Vec3, // 096
    world_acceleration: Vec3,  // 108
    interval_type: u32,        // 120
    interval_value: f32,       // 124
    size_range: Vec2,          // 128
    lifetime_range: Vec2,      // 136
    start_age_range: Vec2,     // 144
    deviation_distance: f32,   // 152
    zero156: Vec2,             // 156
    fade_range: Vec2,          // 164
    friction: f32,             // 172
    zero176: u32,              // 176
    zero180: u32,              // 180
    zero184: u32,              // 184
    zero188: u32,              // 188
    tex192: [u8; 36],          // 192
    tex228: [u8; 36],          // 228
    tex264: [u8; 36],          // 264
    tex300: [u8; 36],          // 300
    tex336: [u8; 36],          // 336
    tex372: [u8; 36],          // 372
    zero408: [u8; 120],        // 408
    unk528: u32,               // 528
    zero532: u32,              // 532
    unk536: f32,               // 536
    unk540: f32,               // 540
    growth_factor: f32,        // 544
    zero548: [u8; 32],         // 548
}
static_assert_size!(PufferStateC, 580);

bitflags::bitflags! {
    struct PufferStateFlags: u32 {
        const INACTIVE = 0;
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

#[derive(Debug, Serialize, Deserialize)]
pub enum IntervalType {
    Unset,
    Time,
    Distance,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Interval {
    pub interval_type: IntervalType,
    pub interval_value: f32,
    pub flag: bool,
}

pub type PufferStateTextures = (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

#[derive(Debug, Serialize, Deserialize)]
pub struct PufferState {
    pub name: String,
    pub state: bool,
    pub translate: bool,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub active_state: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub at_node: Option<AtNode>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub local_velocity: Option<Vec3>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub world_velocity: Option<Vec3>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub min_random_velocity: Option<Vec3>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub max_random_velocity: Option<Vec3>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub world_acceleration: Option<Vec3>,
    pub interval: Interval,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub size_range: Option<Vec2>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub lifetime_range: Option<Vec2>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub start_age_range: Option<Vec2>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub deviation_distance: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub fade_range: Option<Vec2>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub friction: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub textures: Option<PufferStateTextures>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub growth_factor: Option<f32>,
}

impl ScriptObject for PufferState {
    const INDEX: u8 = 42;
    const SIZE: u32 = PufferStateC::SIZE;

    fn read<R: Read>(read: &mut CountingReader<R>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("puffer state size", size == Self::SIZE, read.offset)?;
        let puffer_state: PufferStateC = read.read_struct()?;
        let name = assert_utf8("puffer state name", read.prev + 0, || {
            str_from_c_padded(&puffer_state.name)
        })?;
        let expected_name =
            anim_def.puffer_from_index(puffer_state.puffer_index as usize, read.prev + 32)?;
        let name_ref = &name;
        assert_that!(
            "puffer state name",
            name_ref == &expected_name,
            read.prev + 0
        )?;
        let flags = PufferStateFlags::from_bits(puffer_state.flags).ok_or_else(|| {
            AssertionError(format!(
                "Expected valid puffer state flags, but was 0x{:08X} (at {})",
                puffer_state.flags,
                read.prev + 36
            ))
        })?;

        let state = flags.contains(PufferStateFlags::STATE);
        if !state {
            // if the puffer state is disabled/inactive, then nothing else may be
            // specified. this ensures all further branches check for zero values.
            assert_that!(
                "puffer state flags",
                puffer_state.flags == 0,
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
                puffer_state.interval_type == 0,
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
            let size_range_min = puffer_state.size_range.0;
            let size_range_max = puffer_state.size_range.1;
            assert_that!(
                "puffer state size range min",
                size_range_min > 0.0,
                read.prev + 128
            )?;
            assert_that!(
                "puffer state size range max",
                size_range_max > size_range_min,
                read.prev + 132
            )?;
            Some(puffer_state.size_range)
        } else {
            assert_that!(
                "puffer state size range",
                puffer_state.size_range == Vec2::DEFAULT,
                read.prev + 128
            )?;
            None
        };

        let lifetime_range = if flags.contains(PufferStateFlags::LIFETIME_RANGE) {
            let lifetime_range_min = puffer_state.lifetime_range.0;
            let lifetime_range_max = puffer_state.lifetime_range.1;
            assert_that!(
                "puffer state lifetime range min",
                lifetime_range_min > 0.0,
                read.prev + 136
            )?;
            // TODO: this doesn't follow ordering?
            assert_that!(
                "puffer state lifetime range max",
                lifetime_range_max > 0.0,
                read.prev + 140
            )?;
            Some(puffer_state.lifetime_range)
        } else {
            assert_that!(
                "puffer state lifetime range",
                puffer_state.lifetime_range == Vec2::DEFAULT,
                read.prev + 136
            )?;
            None
        };

        let start_age_range = if flags.contains(PufferStateFlags::START_AGE_RANGE) {
            let start_age_range_min = puffer_state.start_age_range.0;
            let start_age_range_max = puffer_state.start_age_range.1;
            assert_that!(
                "puffer state start age range min",
                start_age_range_min >= 0.0,
                read.prev + 144
            )?;
            assert_that!(
                "puffer state start age range max",
                start_age_range_max > start_age_range_min,
                read.prev + 148
            )?;
            Some(puffer_state.start_age_range)
        } else {
            assert_that!(
                "puffer state start age range",
                puffer_state.start_age_range == Vec2::DEFAULT,
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
            puffer_state.zero156 == Vec2::DEFAULT,
            read.prev + 156
        )?;

        let fade_range = if flags.contains(PufferStateFlags::FADE_RANGE) {
            let fade_range_min = puffer_state.fade_range.0;
            let fade_range_max = puffer_state.fade_range.1;
            assert_that!(
                "puffer state fade range min",
                fade_range_min > 0.0,
                read.prev + 164
            )?;
            assert_that!(
                "puffer state fade range max",
                fade_range_max > fade_range_min,
                read.prev + 168
            )?;
            Some(puffer_state.fade_range)
        } else {
            assert_that!(
                "puffer state fade range",
                puffer_state.fade_range == Vec2::DEFAULT,
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
            let tex1 = if puffer_state.tex192[0] != 0 {
                Some(assert_utf8(
                    "puffer state texture 192",
                    read.prev + 192,
                    || str_from_c_padded(&puffer_state.tex192),
                )?)
            } else {
                assert_all_zero(
                    "puffer state texture 192",
                    read.prev + 192,
                    &puffer_state.tex192,
                )?;
                None
            };
            let tex2 = if puffer_state.tex228[0] != 0 {
                Some(assert_utf8(
                    "puffer state texture 228",
                    read.prev + 228,
                    || str_from_c_padded(&puffer_state.tex228),
                )?)
            } else {
                assert_all_zero(
                    "puffer state texture 228",
                    read.prev + 228,
                    &puffer_state.tex228,
                )?;
                None
            };
            let tex3 = if puffer_state.tex264[0] != 0 {
                Some(assert_utf8(
                    "puffer state texture 264",
                    read.prev + 264,
                    || str_from_c_padded(&puffer_state.tex264),
                )?)
            } else {
                assert_all_zero(
                    "puffer state texture 264",
                    read.prev + 264,
                    &puffer_state.tex264,
                )?;
                None
            };
            let tex4 = if puffer_state.tex300[0] != 0 {
                Some(assert_utf8(
                    "puffer state texture 300",
                    read.prev + 300,
                    || str_from_c_padded(&puffer_state.tex300),
                )?)
            } else {
                assert_all_zero(
                    "puffer state texture 300",
                    read.prev + 300,
                    &puffer_state.tex300,
                )?;
                None
            };
            let tex5 = if puffer_state.tex336[0] != 0 {
                Some(assert_utf8(
                    "puffer state texture 336",
                    read.prev + 336,
                    || str_from_c_padded(&puffer_state.tex336),
                )?)
            } else {
                assert_all_zero(
                    "puffer state texture 336",
                    read.prev + 336,
                    &puffer_state.tex336,
                )?;
                None
            };
            let tex6 = if puffer_state.tex372[0] != 0 {
                Some(assert_utf8(
                    "puffer state texture 372",
                    read.prev + 372,
                    || str_from_c_padded(&puffer_state.tex372),
                )?)
            } else {
                assert_all_zero(
                    "puffer state texture 372",
                    read.prev + 372,
                    &puffer_state.tex372,
                )?;
                None
            };
            Some((tex1, tex2, tex3, tex4, tex5, tex6))
        } else {
            assert_all_zero(
                "puffer state texture 192",
                read.prev + 192,
                &puffer_state.tex192,
            )?;
            assert_all_zero(
                "puffer state texture 228",
                read.prev + 228,
                &puffer_state.tex228,
            )?;
            assert_all_zero(
                "puffer state texture 264",
                read.prev + 264,
                &puffer_state.tex264,
            )?;
            assert_all_zero(
                "puffer state texture 300",
                read.prev + 300,
                &puffer_state.tex300,
            )?;
            assert_all_zero(
                "puffer state texture 336",
                read.prev + 336,
                &puffer_state.tex336,
            )?;
            assert_all_zero(
                "puffer state texture 372",
                read.prev + 372,
                &puffer_state.tex372,
            )?;
            None
        };

        assert_all_zero(
            "puffer state field 408",
            read.prev + 408,
            &puffer_state.zero408,
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
        assert_all_zero(
            "puffer state field 548",
            read.prev + 548,
            &puffer_state.zero548,
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

    fn write<W: Write>(&self, write: &mut W, anim_def: &AnimDef) -> Result<()> {
        let mut name = [0; 32];
        str_to_c_padded(&self.name, &mut name);
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
            IntervalType::Unset => 0,
            IntervalType::Distance => {
                flags |= PufferStateFlags::INTERVAL_TYPE;
                1
            }
            IntervalType::Time => {
                flags |= PufferStateFlags::INTERVAL_TYPE;
                0
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
        let mut tex192 = [0; 36];
        let mut tex228 = [0; 36];
        let mut tex264 = [0; 36];
        let mut tex300 = [0; 36];
        let mut tex336 = [0; 36];
        let mut tex372 = [0; 36];
        if let Some(textures) = &self.textures {
            flags |= PufferStateFlags::CYCLE_TEXTURE;
            if let Some(tex) = &textures.0 {
                str_to_c_padded(tex, &mut tex192);
            }
            if let Some(tex) = &textures.1 {
                str_to_c_padded(tex, &mut tex228);
            }
            if let Some(tex) = &textures.2 {
                str_to_c_padded(tex, &mut tex264);
            }
            if let Some(tex) = &textures.3 {
                str_to_c_padded(tex, &mut tex300);
            }
            if let Some(tex) = &textures.4 {
                str_to_c_padded(tex, &mut tex336);
            }
            if let Some(tex) = &textures.5 {
                str_to_c_padded(tex, &mut tex372);
            }
        }
        write.write_struct(&PufferStateC {
            name,
            puffer_index,
            flags: flags.bits(),
            active_state: self.active_state.unwrap_or(-1),
            node_index,
            translation,
            local_velocity: self.local_velocity.unwrap_or(Vec3::DEFAULT),
            world_velocity: self.world_velocity.unwrap_or(Vec3::DEFAULT),
            min_random_velocity: self.min_random_velocity.unwrap_or(Vec3::DEFAULT),
            max_random_velocity: self.max_random_velocity.unwrap_or(Vec3::DEFAULT),
            world_acceleration: self.world_acceleration.unwrap_or(Vec3::DEFAULT),
            interval_type,
            interval_value: self.interval.interval_value,
            size_range: self.size_range.unwrap_or(Vec2::DEFAULT),
            lifetime_range: self.lifetime_range.unwrap_or(Vec2::DEFAULT),
            start_age_range: self.start_age_range.unwrap_or(Vec2::DEFAULT),
            deviation_distance: self.deviation_distance.unwrap_or(0.0),
            zero156: Vec2::DEFAULT,
            fade_range: self.fade_range.unwrap_or(Vec2::DEFAULT),
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
            zero408: [0; 120],
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
            zero548: [0; 32],
        })?;
        Ok(())
    }
}
