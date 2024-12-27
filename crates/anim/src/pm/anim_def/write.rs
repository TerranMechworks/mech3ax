use super::{AnimDefC, AnimDefFlags};
use crate::common::activation_prereq::write_activ_prereqs;
use crate::common::fixup::Rev;
// use mech3ax_api_types::anim::events::EventData;
use crate::common::seq_def::{write_reset_state_pm, write_sequence_defs, WriteEventsPm};
use crate::common::support::{
    write_anim_refs, write_dynamic_sounds, write_lights, write_puffers, write_static_sounds,
};
use crate::pm::support::{write_nodes, write_objects};
use mech3ax_anim_names::pm::{anim_name_rev, anim_root_name_rev};
use mech3ax_api_types::anim::{AnimDef, Execution};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{assert_len, assert_with_msg, Result};
use mech3ax_types::{Ascii, Ptr, Zeros};
use std::io::Write;

macro_rules! opt_to_ptr {
    ($opt:expr) => {
        if $opt.is_none() {
            Ptr::NULL
        } else {
            Ptr::INVALID
        }
    };
}

pub(crate) fn write_anim_def(
    write: &mut CountingWriter<impl Write>,
    anim_def: &AnimDef,
) -> Result<()> {
    let ptrs = anim_def.ptrs.as_ref();

    let rev = Rev::new("anim def anim name", anim_name_rev);
    let hash = ptrs.and_then(|ptrs| ptrs.anim_hash);
    let anim_name = rev.fixup(&anim_def.anim_name, hash);
    let name = Ascii::from_str_padded(&anim_def.name);
    let rev = Rev::new("anim def anim root name", anim_root_name_rev);
    let hash = ptrs.and_then(|ptrs| ptrs.anim_root_hash);
    let anim_root_name = rev.fixup(&anim_def.anim_root_name, hash);

    let status = if anim_def.active { 0 } else { 5 };

    let mut flags = AnimDefFlags::empty();
    if let Some(network_log_on) = anim_def.network_log {
        flags |= AnimDefFlags::NETWORK_LOG_SET;
        if network_log_on {
            flags |= AnimDefFlags::NETWORK_LOG_ON;
        }
    }
    if let Some(save_log_on) = anim_def.save_log {
        flags |= AnimDefFlags::SAVE_LOG_SET;
        if save_log_on {
            flags |= AnimDefFlags::SAVE_LOG_ON;
        }
    }

    let (exec_by_range_min, exec_by_range_max) = match anim_def.execution {
        Execution::None => (0.0, 0.0),
        Execution::ByZone => {
            flags |= AnimDefFlags::EXECUTION_BY_ZONE;
            (0.0, 0.0)
        }
        Execution::ByRange(range) => {
            flags |= AnimDefFlags::EXECUTION_BY_RANGE;
            (range.min, range.max)
        }
    };

    if anim_def.reset_time.is_some() {
        flags |= AnimDefFlags::RESET_TIME;
    }
    if anim_def.has_callbacks {
        flags |= AnimDefFlags::HAS_CALLBACKS;
    }
    if anim_def.auto_reset_node_states {
        flags |= AnimDefFlags::AUTO_RESET_NODE_STATES;
    }
    if anim_def.local_nodes_only {
        flags |= AnimDefFlags::LOCAL_NODES_ONLY;
    }
    if anim_def.proximity_damage {
        flags |= AnimDefFlags::PROXIMITY_DAMAGE;
    }

    let seq_def_count = assert_len!(u8, anim_def.sequences.len(), "anim def sequence length")?;
    let object_count = anim_def
        .objects
        .as_ref()
        .map(|v| assert_len!(u8, v.len() + 1, "anim def objects length"))
        .transpose()?
        .unwrap_or(0);
    let node_count = anim_def
        .nodes
        .as_ref()
        .map(|v| assert_len!(u8, v.len() + 1, "anim def nodes length"))
        .transpose()?
        .unwrap_or(0);
    let light_count = anim_def
        .lights
        .as_ref()
        .map(|v| assert_len!(u8, v.len() + 1, "anim def lights length"))
        .transpose()?
        .unwrap_or(0);
    let puffer_count = anim_def
        .puffers
        .as_ref()
        .map(|v| assert_len!(u8, v.len() + 1, "anim def puffers length"))
        .transpose()?
        .unwrap_or(0);
    let dynamic_sound_count = anim_def
        .dynamic_sounds
        .as_ref()
        .map(|v| assert_len!(u8, v.len() + 1, "anim def dynamic sounds length"))
        .transpose()?
        .unwrap_or(0);
    let static_sound_count = anim_def
        .static_sounds
        .as_ref()
        .map(|v| assert_len!(u8, v.len() + 1, "anim def static sounds length"))
        .transpose()?
        .unwrap_or(0);
    if anim_def.effects.is_some() {
        return Err(assert_with_msg!("Anim def (pm) does not support effects"));
    }
    let activ_prereq_count = anim_def
        .activ_prereqs
        .as_ref()
        .map(|v| assert_len!(u8, v.len(), "anim def activation prerequisites length"))
        .transpose()?
        .unwrap_or(0);
    let anim_ref_count = anim_def
        .anim_refs
        .as_ref()
        .map(|v| assert_len!(u8, v.len(), "anim def anim ref length"))
        .transpose()?
        .unwrap_or(0);

    let seq_defs_ptr = ptrs
        .map(|ptrs| Ptr(ptrs.seq_defs_ptr))
        .unwrap_or(Ptr::INVALID);
    let reset_state_ptr = ptrs
        .map(|ptrs| Ptr(ptrs.reset_state_ptr))
        .unwrap_or(opt_to_ptr!(anim_def.reset_state));

    let objects_ptr = ptrs
        .map(|ptrs| Ptr(ptrs.objects_ptr))
        .unwrap_or(opt_to_ptr!(anim_def.objects));
    let nodes_ptr = ptrs
        .map(|ptrs| Ptr(ptrs.nodes_ptr))
        .unwrap_or(opt_to_ptr!(anim_def.nodes));
    let lights_ptr = ptrs
        .map(|ptrs| Ptr(ptrs.lights_ptr))
        .unwrap_or(opt_to_ptr!(anim_def.lights));
    let puffers_ptr = ptrs
        .map(|ptrs| Ptr(ptrs.puffers_ptr))
        .unwrap_or(opt_to_ptr!(anim_def.puffers));
    let dynamic_sounds_ptr = ptrs
        .map(|ptrs| Ptr(ptrs.dynamic_sounds_ptr))
        .unwrap_or(opt_to_ptr!(anim_def.dynamic_sounds));
    let static_sounds_ptr = ptrs
        .map(|ptrs| Ptr(ptrs.static_sounds_ptr))
        .unwrap_or(opt_to_ptr!(anim_def.static_sounds));
    let activ_prereqs_ptr = ptrs
        .map(|ptrs| Ptr(ptrs.activ_prereqs_ptr))
        .unwrap_or(opt_to_ptr!(anim_def.activ_prereqs));
    let anim_refs_ptr = ptrs
        .map(|ptrs| Ptr(ptrs.anim_refs_ptr))
        .unwrap_or(opt_to_ptr!(anim_def.anim_refs));

    let anim_def_c = AnimDefC {
        anim_name,
        unknowns_ptr: Ptr::NULL,
        unknowns_count: Ptr::NULL,
        name,
        anim_ptr: Ptr::INVALID,
        anim_root_name,
        anim_root_ptr: Ptr::INVALID,
        zero112: Zeros::new(),
        flags: flags.maybe(),
        status,
        activation: anim_def.activation.maybe(),
        execution_priority: 4,
        two163: 2,
        exec_by_range_min,
        exec_by_range_max,
        reset_time: anim_def.reset_time.unwrap_or(-1.0),
        zero176: 0.0,
        max_health: anim_def.health,
        cur_health: anim_def.health,
        zero188: 0,
        zero192: 0,
        zero196: 0,
        zero200: 0,
        seq_defs_ptr,
        reset_state_ptr,
        unknown_seq_ptr: Ptr::NULL,
        seq_def_count,
        object_count,
        node_count,
        light_count,
        puffer_count,
        dynamic_sound_count,
        static_sound_count,
        effect_count: 0,
        activ_prereq_count,
        activ_prereq_min_to_satisfy: anim_def.activ_prereq_min_to_satisfy,
        anim_ref_count,
        zero227: 0,
        objects_ptr,
        nodes_ptr,
        lights_ptr,
        puffers_ptr,
        dynamic_sounds_ptr,
        static_sounds_ptr,
        effects_ptr: Ptr::NULL,
        activ_prereqs_ptr,
        anim_refs_ptr,
        zero264: 0,
    };
    write.write_struct(&anim_def_c)?;

    if let Some(objects) = &anim_def.objects {
        write_objects(write, objects)?;
    }
    if let Some(nodes) = &anim_def.nodes {
        write_nodes(write, nodes)?;
    }
    if let Some(lights) = &anim_def.lights {
        write_lights(write, lights)?;
    }
    if let Some(puffers) = &anim_def.puffers {
        write_puffers(write, puffers)?;
    }
    if let Some(dynamic_sounds) = &anim_def.dynamic_sounds {
        write_dynamic_sounds(write, dynamic_sounds)?;
    }
    if let Some(static_sounds) = &anim_def.static_sounds {
        write_static_sounds(write, static_sounds)?;
    }
    if let Some(activ_prereqs) = &anim_def.activ_prereqs {
        write_activ_prereqs(write, activ_prereqs, true)?;
    }
    if let Some(anim_refs) = &anim_def.anim_refs {
        write_anim_refs(write, anim_refs)?;
    }
    if let Some(reset_state) = &anim_def.reset_state {
        write_reset_state_pm(write, anim_def, reset_state)?;
    }
    let write_events = WriteEventsPm;
    write_sequence_defs(write, anim_def, write_events)?;

    Ok(())
}
