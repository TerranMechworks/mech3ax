use super::{
    PufferStateColors, PufferStateCommon, PufferStateFlags, PufferStateGrowths,
    PufferStateTextureC, PufferStateTextures,
};
use crate::types::{AnimDefLookup as _, index};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::anim::events::{
    PufferIntervalType, PufferState, PufferStateColor, PufferStateTexture,
};
use mech3ax_api_types::{Range, Vec3};
use mech3ax_common::{Result, assert_len, assert_with_msg};
use mech3ax_types::{Ascii, Bool32};

pub(super) fn make_common(state: &PufferState, anim_def: &AnimDef) -> Result<PufferStateCommon> {
    let puffer_name = Ascii::from_str_padded(&state.name);
    let puffer_index = anim_def.puffer_to_index(&state.name)?;
    let mut flags = PufferStateFlags::empty();
    if state.translate.is_some() {
        flags |= PufferStateFlags::TRANSLATE_ABS;
    }
    let node_index = if let Some(node) = &state.at_node {
        flags |= PufferStateFlags::AT_NODE;
        anim_def.node_to_index(node)?
    } else {
        index!(-1)
    };
    if state.active_state.is_some() {
        flags |= PufferStateFlags::ACTIVE_STATE;
    }
    if state.local_velocity.is_some() {
        flags |= PufferStateFlags::LOCAL_VELOCITY;
    }
    if state.world_velocity.is_some() {
        flags |= PufferStateFlags::WORLD_VELOCITY;
    }
    if state.min_random_velocity.is_some() {
        flags |= PufferStateFlags::MIN_RANDOM_VELOCITY;
    }
    if state.max_random_velocity.is_some() {
        flags |= PufferStateFlags::MAX_RANDOM_VELOCITY;
    }
    let (interval_type, interval_value) = match (&state.interval, &state.interval_garbage) {
        (Some(interval), _) => {
            flags |= PufferStateFlags::INTERVAL_TYPE | PufferStateFlags::INTERVAL_VALUE;
            let interval_type = match interval.interval_type {
                PufferIntervalType::Distance => Bool32::TRUE,
                PufferIntervalType::Time => Bool32::FALSE,
            };
            let interval_value = interval.interval_value;
            (interval_type, interval_value)
        }
        (None, Some(garbage)) => {
            if garbage.has_interval_type {
                flags |= PufferStateFlags::INTERVAL_TYPE;
            }
            if garbage.has_interval_value {
                flags |= PufferStateFlags::INTERVAL_VALUE;
            }
            let interval_type = match garbage.interval_type {
                PufferIntervalType::Distance => Bool32::TRUE,
                PufferIntervalType::Time => Bool32::FALSE,
            };
            let interval_value = garbage.interval_value;
            (interval_type, interval_value)
        }
        (None, None) => (Bool32::FALSE, 0.0),
    };
    if state.size_range.is_some() {
        flags |= PufferStateFlags::SIZE_RANGE;
    }
    if state.lifetime_range.is_some() {
        flags |= PufferStateFlags::LIFETIME_RANGE;
    }
    if state.deviation_distance.is_some() {
        flags |= PufferStateFlags::DEVIATION_DISTANCE;
    }
    if state.fade_range.is_some() {
        flags |= PufferStateFlags::FADE_RANGE;
    }
    if state.growth_factors.is_some() {
        flags |= PufferStateFlags::GROWTH_FACTORS;
    }
    if state.textures.is_some() {
        flags |= PufferStateFlags::TEXTURES;
    }
    if state.start_age_range.is_some() {
        flags |= PufferStateFlags::START_AGE_RANGE;
    }
    if state.world_acceleration.is_some() {
        flags |= PufferStateFlags::WORLD_ACCELERATION;
    }
    if state.friction.is_some() {
        flags |= PufferStateFlags::FRICTION;
    }
    if state.colors.is_some() {
        flags |= PufferStateFlags::COLORS;
    }
    if state.unk_range.is_some() {
        flags |= PufferStateFlags::UNKNOWN_RANGE;
    }
    if state.wind_factor.is_some() {
        flags |= PufferStateFlags::WIND_FACTOR;
    }
    if state.number.is_some() {
        flags |= PufferStateFlags::NUMBER;
    }
    if state.priority.is_some() {
        flags |= PufferStateFlags::PRIORITY;
    }

    Ok(PufferStateCommon {
        puffer_name,
        puffer_index,
        flags: flags.maybe(),
        node_index,
        active_state: state.active_state.unwrap_or(0),
        translate: state.translate.unwrap_or(Vec3::DEFAULT),
        local_velocity: state.local_velocity.unwrap_or(Vec3::DEFAULT),
        world_velocity: state.world_velocity.unwrap_or(Vec3::DEFAULT),
        min_random_velocity: state.min_random_velocity.unwrap_or(Vec3::DEFAULT),
        max_random_velocity: state.max_random_velocity.unwrap_or(Vec3::DEFAULT),
        world_acceleration: state.world_acceleration.unwrap_or(Vec3::DEFAULT),
        interval_type,
        interval_value,
        size_range: state.size_range.unwrap_or(Range::DEFAULT),
        lifetime_range: state.lifetime_range.unwrap_or(Range::DEFAULT),
        start_age_range: state.start_age_range.unwrap_or(Range::DEFAULT),
        deviation_distance: state.deviation_distance.unwrap_or(0.0),
        unk_range: state.unk_range.unwrap_or(Range::DEFAULT),
        fade_range: state.fade_range.unwrap_or(Range::DEFAULT),
        friction: state.friction.unwrap_or(0.0),
        wind_factor: state.wind_factor.unwrap_or(0.0),
        priority: state.priority.unwrap_or(0.0),
    })
}

pub(super) fn make_textures(
    textures: &Option<Vec<PufferStateTexture>>,
) -> Result<PufferStateTextures> {
    let mut t = [PufferStateTextureC::ZERO; 6];
    let mut has_run_time = false;

    if let Some(textures) = textures {
        let count = assert_len!(u32, textures.len(), "puffer state textures")?;
        if count > 6 {
            return Err(assert_with_msg!(
                "`puffer state textures` must be <= 4, but was {}",
                count
            ));
        }

        for (i, texture) in textures.iter().enumerate() {
            if texture.run_time.is_some() {
                has_run_time = true;
            }
            t[i] = PufferStateTextureC {
                name: Ascii::from_str_padded(&texture.name),
                run_time: texture.run_time.unwrap_or(0.0),
            };
        }
    }

    let [
        texture_0,
        texture_1,
        texture_2,
        texture_3,
        texture_4,
        texture_5,
    ] = t;
    Ok(PufferStateTextures {
        has_run_time: has_run_time.into(),
        texture_0,
        texture_1,
        texture_2,
        texture_3,
        texture_4,
        texture_5,
    })
}

pub(super) fn make_colors(colors: &Option<Vec<PufferStateColor>>) -> Result<PufferStateColors> {
    let mut count = 0;
    let mut c = [PufferStateColor::ZERO; 6];

    if let Some(colors) = colors {
        count = assert_len!(u32, colors.len(), "puffer state colors")?;
        if count > 6 {
            return Err(assert_with_msg!(
                "`puffer state colors` must be <= 6, but was {}",
                count
            ));
        }

        for (i, color) in colors.iter().enumerate() {
            c[i] = *color;
        }
    }

    let [color_0, color_1, color_2, color_3, color_4, color_5] = c;
    Ok(PufferStateColors {
        count,
        color_0,
        color_1,
        color_2,
        color_3,
        color_4,
        color_5,
    })
}

pub(super) fn make_growths(growth_factors: &Option<Vec<Range>>) -> Result<PufferStateGrowths> {
    let mut count = 0;
    let mut g = [Range::DEFAULT; 6];

    if let Some(growths) = growth_factors {
        count = assert_len!(u32, growths.len(), "puffer state growth factors")?;
        if count > 6 {
            return Err(assert_with_msg!(
                "`puffer state growth factors` must be <= 6, but was {}",
                count
            ));
        }

        for (i, growth) in growths.iter().enumerate() {
            g[i] = *growth;
        }
    }

    let [growth_0, growth_1, growth_2, growth_3, growth_4, growth_5] = g;
    Ok(PufferStateGrowths {
        count,
        growth_0,
        growth_1,
        growth_2,
        growth_3,
        growth_4,
        growth_5,
    })
}
