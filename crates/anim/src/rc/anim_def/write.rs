use super::{AnimDefC, AnimDefFlags, RESET_TIME_BORK};
use crate::common::activation_prereq::write_activ_prereqs;
use crate::common::fixup::Rev;
use crate::common::seq_def::{
    SeqDefInfoC, WriteEventsRc, write_reset_state_pg, write_sequence_defs,
};
use crate::common::support::{
    write_anim_refs, write_dynamic_sounds, write_effects, write_lights, write_nodes, write_objects,
    write_static_sounds,
};
use log::debug;
use mech3ax_anim_events::rc::size_events;
use mech3ax_anim_names::rc::{anim_name_rev, anim_root_name_rev};
use mech3ax_api_types::anim::{AnimDef, Execution, SiScript};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{Result, assert_len, assert_with_msg};
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
    scripts: &[SiScript],
) -> Result<()> {
    let ptrs = anim_def.ptrs.as_ref();

    let rev = Rev::new("anim def anim name", anim_name_rev);
    let hash = ptrs.and_then(|ptrs| ptrs.anim_hash);
    let anim_name = rev.fixup(&anim_def.anim_name, hash);
    let name = Ascii::from_str_padded(&anim_def.name);
    let rev = Rev::new("anim def anim root name", anim_root_name_rev);
    let hash = ptrs.and_then(|ptrs| ptrs.anim_root_hash);
    let anim_root_name = rev.fixup(&anim_def.anim_root_name, hash);

    if !anim_def.active {
        return Err(assert_with_msg!(
            "Anim def (rc) does not support active == false"
        ));
    }

    let mut flags = AnimDefFlags::empty();
    if let Some(true) = anim_def.network_log {
        flags |= AnimDefFlags::NETWORK_LOG;
    }
    if let Some(true) = anim_def.save_log {
        flags |= AnimDefFlags::SAVE_LOG;
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
        return Err(assert_with_msg!(
            "Anim def (rc) does not support auto_reset_node_states"
        ));
    }
    if anim_def.local_nodes_only {
        return Err(assert_with_msg!(
            "Anim def (rc) does not support local_nodes_only"
        ));
    }
    if anim_def.proximity_damage {
        return Err(assert_with_msg!(
            "Anim def (rc) does not support proximity_damage"
        ));
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
    if anim_def.puffers.is_some() {
        return Err(assert_with_msg!("Anim def (rc) does not support puffers"));
    }
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
    let effect_count = anim_def
        .effects
        .as_ref()
        .map(|v| assert_len!(u8, v.len() + 1, "anim def effects length"))
        .transpose()?
        .unwrap_or(0);
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

    let reset_state_pointer = anim_def
        .reset_state
        .as_ref()
        .map(|state| state.pointer)
        .unwrap_or(0);
    let reset_state_size = anim_def
        .reset_state
        .as_ref()
        .map(|state| {
            size_events(&state.events, scripts)
                .ok_or_else(|| assert_with_msg!("Reset state event data overflow"))
        })
        .transpose()?
        .unwrap_or(0);

    let reset_state = SeqDefInfoC::make_reset_state(reset_state_pointer, reset_state_size);

    let execution_priority = if anim_def.low_priority { 1 } else { 4 };

    let anim_ptr = ptrs.map(|ptrs| Ptr(ptrs.anim_ptr)).unwrap_or(Ptr::INVALID);
    let anim_root_ptr = ptrs
        .map(|ptrs| Ptr(ptrs.anim_root_ptr))
        .unwrap_or(Ptr::INVALID);
    let seq_defs_ptr = ptrs
        .map(|ptrs| Ptr(ptrs.seq_defs_ptr))
        .unwrap_or(Ptr::INVALID);

    let objects_ptr = ptrs
        .map(|ptrs| Ptr(ptrs.objects_ptr))
        .unwrap_or(opt_to_ptr!(anim_def.objects));
    let nodes_ptr = ptrs
        .map(|ptrs| Ptr(ptrs.nodes_ptr))
        .unwrap_or(opt_to_ptr!(anim_def.nodes));
    let lights_ptr = ptrs
        .map(|ptrs| Ptr(ptrs.lights_ptr))
        .unwrap_or(opt_to_ptr!(anim_def.lights));
    let dynamic_sounds_ptr = ptrs
        .map(|ptrs| Ptr(ptrs.dynamic_sounds_ptr))
        .unwrap_or(opt_to_ptr!(anim_def.dynamic_sounds));
    let static_sounds_ptr = ptrs
        .map(|ptrs| Ptr(ptrs.static_sounds_ptr))
        .unwrap_or(opt_to_ptr!(anim_def.static_sounds));
    let effects_ptr = ptrs
        .map(|ptrs| Ptr(ptrs.effects_ptr))
        .unwrap_or(opt_to_ptr!(anim_def.effects));
    let activ_prereqs_ptr = ptrs
        .map(|ptrs| Ptr(ptrs.activ_prereqs_ptr))
        .unwrap_or(opt_to_ptr!(anim_def.activ_prereqs));
    let anim_refs_ptr = ptrs
        .map(|ptrs| Ptr(ptrs.anim_refs_ptr))
        .unwrap_or(opt_to_ptr!(anim_def.anim_refs));

    let reset_time = anim_def.reset_time.unwrap_or_else(|| {
        if RESET_TIME_BORK.contains(&seq_defs_ptr.0) {
            debug!("anim def reset time == 0.0 fixup");
            0.0
        } else {
            -1.0
        }
    });

    let anim_def_c = AnimDefC {
        anim_name,
        name,
        anim_ptr,
        anim_root_name,
        anim_root_ptr,
        zero104: Zeros::new(),
        flags: flags.maybe(),
        status: 0,
        activation: anim_def.activation.maybe(),
        execution_priority,
        two155: 2,
        exec_by_range_min,
        exec_by_range_max,
        reset_time,
        zero168: 0.0,
        max_health: anim_def.health,
        cur_health: anim_def.health,
        zero180: 0,
        zero184: 0,
        zero188: 0,
        seq_defs_ptr,
        reset_state,
        seq_def_count,
        object_count,
        node_count,
        light_count,
        dynamic_sound_count,
        static_sound_count,
        effect_count,
        activ_prereq_count,
        activ_prereq_min_to_satisfy: anim_def.activ_prereq_min_to_satisfy,
        anim_ref_count,
        zero270: 0,
        zero271: 0,
        objects_ptr,
        nodes_ptr,
        lights_ptr,
        dynamic_sounds_ptr,
        static_sounds_ptr,
        effects_ptr,
        activ_prereqs_ptr,
        anim_refs_ptr,
        zero304: 0,
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
    if let Some(dynamic_sounds) = &anim_def.dynamic_sounds {
        write_dynamic_sounds(write, dynamic_sounds)?;
    }
    if let Some(static_sounds) = &anim_def.static_sounds {
        write_static_sounds(write, static_sounds)?;
    }
    if let Some(effects) = &anim_def.effects {
        write_effects(write, effects)?;
    }
    if let Some(activ_prereqs) = &anim_def.activ_prereqs {
        write_activ_prereqs(write, activ_prereqs, false)?;
    }
    if let Some(anim_refs) = &anim_def.anim_refs {
        write_anim_refs(write, anim_refs)?;
    }
    let write_events = WriteEventsRc { scripts };
    write_reset_state_pg(write, anim_def, reset_state_size, write_events)?;
    let write_events = WriteEventsRc { scripts };
    write_sequence_defs(write, anim_def, write_events)?;

    Ok(())
}
