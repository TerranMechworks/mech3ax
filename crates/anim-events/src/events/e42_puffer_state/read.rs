use super::{
    PufferStateColors, PufferStateCommon, PufferStateFlags, PufferStateGrowths,
    PufferStateTextureC, PufferStateTextures,
};
use crate::types::{AnimDefLookup as _, index};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::anim::events::{
    PufferInterval, PufferIntervalGarbage, PufferIntervalType, PufferState, PufferStateColor,
    PufferStateTexture,
};
use mech3ax_api_types::{Range, Vec3};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::{Result, assert_that};
use mech3ax_types::{AsBytes as _, u32_to_usize};

pub(super) fn assert_common(
    common: PufferStateCommon,
    anim_def: &AnimDef,
    offset: usize,
) -> Result<(PufferStateFlags, PufferState)> {
    let name = assert_utf8("puffer state name", offset + 0, || {
        common.puffer_name.to_str_padded()
    })?;
    let expected_name = anim_def.puffer_from_index(common.puffer_index, offset + 32)?;
    assert_that!("puffer state name", name == expected_name, offset + 0)?;
    let flags = assert_that!("puffer state flags", flags common.flags, offset + 36)?;

    let has_active_state = flags.contains(PufferStateFlags::ACTIVE_STATE);
    if !has_active_state {
        // if the puffer state is disabled/inactive, then nothing else may be
        // specified. this ensures all further branches check for zero values.
        assert_that!(
            "puffer state flags",
            flags == PufferStateFlags::empty(),
            offset + 36
        )?;
    }

    let active_state = if has_active_state {
        assert_that!("puffer state active", 0 <= common.active_state <= 10, offset + 44)?;
        Some(common.active_state)
    } else {
        assert_that!("puffer state active", common.active_state == 0, offset + 44)?;
        None
    };

    let at_node = if flags.contains(PufferStateFlags::AT_NODE) {
        let node = anim_def.node_from_index(common.node_index, offset + 40)?;
        Some(node)
    } else {
        assert_that!(
            "puffer state node index",
            common.node_index == index!(-1),
            offset + 40
        )?;
        None
    };

    let translate = if flags.contains(PufferStateFlags::TRANSLATE_ABS) {
        Some(common.translate)
    } else {
        assert_that!(
            "puffer state translate",
            common.translate == Vec3::DEFAULT,
            offset + 48
        )?;
        None
    };

    let local_velocity = if flags.contains(PufferStateFlags::LOCAL_VELOCITY) {
        Some(common.local_velocity)
    } else {
        assert_that!(
            "puffer state local velocity",
            common.local_velocity == Vec3::DEFAULT,
            offset + 60
        )?;
        None
    };

    let world_velocity = if flags.contains(PufferStateFlags::WORLD_VELOCITY) {
        Some(common.world_velocity)
    } else {
        assert_that!(
            "puffer state world velocity",
            common.world_velocity == Vec3::DEFAULT,
            offset + 72
        )?;
        None
    };

    let min_random_velocity = if flags.contains(PufferStateFlags::MIN_RANDOM_VELOCITY) {
        Some(common.min_random_velocity)
    } else {
        assert_that!(
            "puffer state min rnd velocity",
            common.min_random_velocity == Vec3::DEFAULT,
            offset + 84
        )?;
        None
    };

    let max_random_velocity = if flags.contains(PufferStateFlags::MAX_RANDOM_VELOCITY) {
        Some(common.max_random_velocity)
    } else {
        assert_that!(
            "puffer state max rnd velocity",
            common.max_random_velocity == Vec3::DEFAULT,
            offset + 96
        )?;
        None
    };

    let world_acceleration = if flags.contains(PufferStateFlags::WORLD_ACCELERATION) {
        Some(common.world_acceleration)
    } else {
        assert_that!(
            "puffer state world accel",
            common.world_acceleration == Vec3::DEFAULT,
            offset + 108
        )?;
        None
    };

    let interval_type =
        assert_that!("puffer state interval type", bool common.interval_type, offset + 120)?;
    let interval_type = if interval_type {
        PufferIntervalType::Distance
    } else {
        PufferIntervalType::Time
    };

    let has_interval_type = flags.contains(PufferStateFlags::INTERVAL_TYPE);
    let has_interval_value = flags.contains(PufferStateFlags::INTERVAL_VALUE);

    let (interval, interval_garbage) = if has_interval_type && has_interval_value {
        log::trace!("puffer state interval: OK");
        assert_that!(
            "puffer state interval value",
            common.interval_value >= 0.0,
            offset + 124
        )?;
        let interval = PufferInterval {
            interval_type,
            interval_value: common.interval_value,
        };
        (Some(interval), None)
    } else {
        log::debug!("puffer state interval: FAIL");
        log::error!(
            "INTERVAL VAL FAIL: `{}`, `{}`",
            anim_def.anim_name,
            anim_def.anim_root_name
        );
        let garbage = PufferIntervalGarbage {
            interval_type,
            has_interval_type,
            interval_value: common.interval_value,
            has_interval_value,
        };
        log::debug!("{:#?}", garbage);
        (None, Some(garbage))
    };

    let size_range = if flags.contains(PufferStateFlags::SIZE_RANGE) {
        assert_that!(
            "puffer state size range min",
            common.size_range.min > 0.0,
            offset + 128
        )?;
        assert_that!(
            "puffer state size range max",
            common.size_range.max > common.size_range.min,
            offset + 132
        )?;
        Some(common.size_range)
    } else {
        assert_that!(
            "puffer state size range",
            common.size_range == Range::DEFAULT,
            offset + 128
        )?;
        None
    };

    let lifetime_range = if flags.contains(PufferStateFlags::LIFETIME_RANGE) {
        assert_that!(
            "puffer state lifetime range min",
            common.lifetime_range.min > 0.0,
            offset + 136
        )?;
        // `trailpuffer2` sometimes has max < min
        assert_that!(
            "puffer state lifetime range max",
            common.lifetime_range.max > 0.0,
            offset + 140
        )?;
        Some(common.lifetime_range)
    } else {
        assert_that!(
            "puffer state lifetime range",
            common.lifetime_range == Range::DEFAULT,
            offset + 136
        )?;
        None
    };

    let start_age_range = if flags.contains(PufferStateFlags::START_AGE_RANGE) {
        assert_that!(
            "puffer state start age range min",
            common.start_age_range.min >= 0.0,
            offset + 144
        )?;
        assert_that!(
            "puffer state start age range max",
            common.start_age_range.max > common.start_age_range.min,
            offset + 148
        )?;
        Some(common.start_age_range)
    } else {
        assert_that!(
            "puffer state start age range",
            common.start_age_range == Range::DEFAULT,
            offset + 144
        )?;
        None
    };

    let deviation_distance = if flags.contains(PufferStateFlags::DEVIATION_DISTANCE) {
        assert_that!(
            "puffer state deviation distance",
            common.deviation_distance > 0.0,
            offset + 152
        )?;
        Some(common.deviation_distance)
    } else {
        assert_that!(
            "puffer state deviation distance",
            common.deviation_distance == 0.0,
            offset + 152
        )?;
        None
    };

    let unk_range = if flags.contains(PufferStateFlags::UNKNOWN_RANGE) {
        // never in games
        assert_that!(
            "puffer state unk range near",
            common.unk_range.min > 0.0,
            offset + 156
        )?;
        assert_that!(
            "puffer state unk range max",
            common.unk_range.max >= common.unk_range.min,
            offset + 160
        )?;
        Some(common.unk_range)
    } else {
        assert_that!(
            "puffer state unk range",
            common.unk_range == Range::DEFAULT,
            offset + 156
        )?;
        None
    };

    let fade_range = if flags.contains(PufferStateFlags::FADE_RANGE) {
        assert_that!(
            "puffer state fade range near",
            common.fade_range.min > 0.0,
            offset + 164
        )?;
        assert_that!(
            "puffer state fade range max",
            common.fade_range.max > common.fade_range.min,
            offset + 168
        )?;
        Some(common.fade_range)
    } else {
        assert_that!(
            "puffer state fade range",
            common.fade_range == Range::DEFAULT,
            offset + 164
        )?;
        None
    };

    let friction = if flags.contains(PufferStateFlags::FRICTION) {
        assert_that!(
            "puffer state friction",
            common.friction >= 0.0,
            offset + 172
        )?;
        Some(common.friction)
    } else {
        assert_that!(
            "puffer state friction",
            common.friction == 0.0,
            offset + 172
        )?;
        None
    };

    let wind_factor = if flags.contains(PufferStateFlags::WIND_FACTOR) {
        assert_that!(
            "puffer state wind factor",
            common.wind_factor >= 0.0,
            offset + 176
        )?;
        Some(common.wind_factor)
    } else {
        assert_that!(
            "puffer state wind factor",
            common.wind_factor == 0.0,
            offset + 176
        )?;
        None
    };

    let priority = if flags.contains(PufferStateFlags::PRIORITY) {
        assert_that!(
            "puffer state priority",
            common.priority >= 0.0,
            offset + 180
        )?;
        Some(common.priority)
    } else {
        assert_that!(
            "puffer state priority",
            common.priority == 0.0,
            offset + 180
        )?;
        None
    };

    let state = PufferState {
        name,
        active_state,
        at_node,
        translate,
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
        unk_range,
        fade_range,
        friction,
        wind_factor,
        priority,
        number: None,
        textures: None,
        colors: None,
        growth_factors: None,
        interval_garbage,
    };
    Ok((flags, state))
}

pub(super) fn assert_textures(
    textures: PufferStateTextures,
    flags: PufferStateFlags,
    mut offset: usize,
) -> Result<Option<Vec<PufferStateTexture>>> {
    const TEXTURE_SIZE: usize = u32_to_usize(PufferStateTextureC::SIZE);

    let PufferStateTextures {
        has_run_time,
        texture_0,
        texture_1,
        texture_2,
        texture_3,
        texture_4,
        texture_5,
    } = textures;
    let count: usize = if !texture_5.name.first_is_zero() {
        6
    } else if !texture_4.name.first_is_zero() {
        5
    } else if !texture_3.name.first_is_zero() {
        4
    } else if !texture_2.name.first_is_zero() {
        3
    } else if !texture_1.name.first_is_zero() {
        2
    } else if !texture_0.name.first_is_zero() {
        1
    } else {
        0
    };
    let textures: [PufferStateTextureC; 6] = [
        texture_0, texture_1, texture_2, texture_3, texture_4, texture_5,
    ];

    let has_run_time =
        assert_that!("puffer state texture has run time", bool has_run_time, offset)?;
    offset += 4;

    if flags.contains(PufferStateFlags::TEXTURES) {
        // the first item must exit... right?
        assert_that!("puffer state texture count", count > 0, offset)?;

        let mut t = Vec::with_capacity(4);
        for (i, texture) in textures.into_iter().enumerate() {
            let name = format!("puffer state texture {}", i);
            if i < count {
                assert_that!(&name, texture != PufferStateTextureC::ZERO, offset)?;
                let run_time = if has_run_time {
                    Some(texture.run_time)
                } else {
                    assert_that!(&name, texture.run_time == 0.0, offset)?;
                    None
                };
                let name = assert_utf8(&name, offset + 4, || texture.name.to_str_padded())?;
                t.push(PufferStateTexture { name, run_time });
            } else {
                assert_that!(&name, texture == PufferStateTextureC::ZERO, offset)?;
            }
            offset += TEXTURE_SIZE;
        }

        Ok(Some(t))
    } else {
        for (i, texture) in textures.into_iter().enumerate() {
            let name = format!("puffer state texture {}", i);
            assert_that!(&name, texture == PufferStateTextureC::ZERO, offset)?;
            offset += TEXTURE_SIZE;
        }
        Ok(None)
    }
}

pub(super) fn assert_colors(
    colors: PufferStateColors,
    flags: PufferStateFlags,
    mut offset: usize,
) -> Result<Option<Vec<PufferStateColor>>> {
    const COLOR_SIZE: usize = u32_to_usize(PufferStateColor::SIZE);

    let PufferStateColors {
        count,
        color_0,
        color_1,
        color_2,
        color_3,
        color_4,
        color_5,
    } = colors;
    let colors: [PufferStateColor; 6] = [color_0, color_1, color_2, color_3, color_4, color_5];

    if flags.contains(PufferStateFlags::COLORS) {
        assert_that!(
            "puffer state color count",
            1 <= count <= 5,
            offset
        )?;
        offset += 4;
        let count = u32_to_usize(count);

        let mut c = Vec::with_capacity(6);
        for (i, color) in colors.into_iter().enumerate() {
            let name = format!("puffer state color {}", i);
            if i < count {
                assert_that!(&name, color != PufferStateColor::ZERO, offset)?;
                c.push(color);
            } else {
                assert_that!(&name, color == PufferStateColor::ZERO, offset)?;
            }
            offset += COLOR_SIZE;
        }

        Ok(Some(c))
    } else {
        assert_that!("puffer state color count", count == 0, offset)?;
        offset += 4;

        for (i, color) in colors.into_iter().enumerate() {
            let name = format!("puffer state color {}", i);
            assert_that!(&name, color == PufferStateColor::ZERO, offset)?;
            offset += COLOR_SIZE;
        }
        Ok(None)
    }
}

pub(super) fn assert_growths(
    growths: PufferStateGrowths,
    flags: PufferStateFlags,
    mut offset: usize,
) -> Result<Option<Vec<Range>>> {
    const RANGE_SIZE: usize = u32_to_usize(Range::SIZE);

    let PufferStateGrowths {
        count,
        growth_0,
        growth_1,
        growth_2,
        growth_3,
        growth_4,
        growth_5,
    } = growths;
    let growths: [Range; 6] = [growth_0, growth_1, growth_2, growth_3, growth_4, growth_5];

    if flags.contains(PufferStateFlags::GROWTH_FACTORS) {
        assert_that!(
            "puffer state growth factor count",
            1 <= count <= 5,
            offset
        )?;
        offset += 4;
        let count = u32_to_usize(count);

        let mut g = Vec::with_capacity(6);
        for (i, growth) in growths.into_iter().enumerate() {
            let name = format!("puffer state growth factor {}", i);
            if i < count {
                assert_that!(&name, growth != Range::DEFAULT, offset)?;
                g.push(growth);
            } else {
                assert_that!(&name, growth == Range::DEFAULT, offset)?;
            }
            offset += RANGE_SIZE;
        }

        Ok(Some(g))
    } else {
        assert_that!("puffer state growth factor count", count == 0, offset)?;
        offset += 4;

        for (i, growth) in growths.into_iter().enumerate() {
            let name = format!("puffer state growth factor {}", i);
            assert_that!(&name, growth == Range::DEFAULT, offset)?;
            offset += RANGE_SIZE;
        }
        Ok(None)
    }
}
