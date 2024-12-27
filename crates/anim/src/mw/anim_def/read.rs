use super::{AnimDefC, AnimDefFlags};
use crate::common::activation_prereq::read_activ_prereqs;
use crate::common::fixup::Fwd;
use crate::common::seq_def::{read_reset_state_pg, read_sequence_defs, ReadEventsMw};
use crate::common::support::{
    read_anim_refs, read_dynamic_sounds, read_lights, read_nodes, read_objects, read_puffers,
    read_static_sounds,
};
use mech3ax_anim_names::mw::{anim_name_fwd, anim_root_name_fwd};
// use mech3ax_api_types::anim::events::EventData;
use mech3ax_api_types::anim::{AnimDef, AnimDefName, Execution, SiScript};
use mech3ax_api_types::Range;
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{assert_that, Result};
use std::io::Read;

pub(crate) fn read_anim_def(
    read: &mut CountingReader<impl Read>,
    scripts: &mut Vec<SiScript>,
) -> Result<(AnimDef, AnimDefName)> {
    let anim_def: AnimDefC = read.read_struct()?;

    // save this so we can output accurate offsets after doing further reads
    let prev = read.prev;

    let fwd = Fwd::new("anim def anim name", anim_name_fwd);
    let (anim_name, anim_hash) = fwd.fixup(prev + 0, &anim_def.anim_name)?;
    let name = assert_utf8("anim def name", prev + 32, || anim_def.name.to_str_padded())?;
    assert_that!("anim def anim ptr", anim_def.anim_ptr != 0, prev + 64)?;
    let fwd = Fwd::new("anim def anim root name", anim_root_name_fwd);
    let (anim_root_name, anim_root_hash) = fwd.fixup(prev + 68, &anim_def.anim_root_name)?;

    let base_name = name.strip_suffix(".flt").unwrap_or(&name);
    let file_name = if name != anim_root_name {
        assert_that!(
            "anim def anim root ptr",
            anim_def.anim_root_ptr != anim_def.anim_ptr,
            prev + 100
        )?;
        format!("{}-{}-{}", base_name, anim_name, anim_root_name)
    } else {
        assert_that!(
            "anim def anim root ptr",
            anim_def.anim_root_ptr == anim_def.anim_ptr,
            prev + 100
        )?;
        format!("{}-{}", base_name, anim_name)
    };

    assert_that!("anim def field 104", zero anim_def.zero104, prev + 104)?;

    let flags = assert_that!("anim def flags", flags anim_def.flags, read.prev + 148)?;

    let network_log = if flags.contains(AnimDefFlags::NETWORK_LOG_SET) {
        Some(flags.contains(AnimDefFlags::NETWORK_LOG_ON))
    } else {
        let network_log_on = flags.contains(AnimDefFlags::NETWORK_LOG_ON);
        assert_that!(
            "anim def network log on",
            network_log_on == false,
            prev + 148
        )?;
        None
    };

    let save_log = if flags.contains(AnimDefFlags::SAVE_LOG_SET) {
        Some(flags.contains(AnimDefFlags::SAVE_LOG_ON))
    } else {
        let save_log_on = flags.contains(AnimDefFlags::SAVE_LOG_ON);
        assert_that!("anim def save log on", save_log_on == false, prev + 148)?;
        None
    };

    assert_that!("anim def status", anim_def.status == 0, prev + 152)?;
    let activation =
        assert_that!("anim def activation", enum anim_def.activation, read.prev + 153)?;
    assert_that!(
        "anim def execution priority",
        anim_def.execution_priority == 4,
        prev + 154
    )?;
    assert_that!("anim def field 155", anim_def.two155 == 2, prev + 155)?;

    let exec_by_zone = flags.contains(AnimDefFlags::EXECUTION_BY_ZONE);
    let execution = if flags.contains(AnimDefFlags::EXECUTION_BY_RANGE) {
        assert_that!("anim def exec by zone", exec_by_zone == false, prev + 148)?;
        assert_that!(
            "anim def exec by range min",
            anim_def.exec_by_range_min >= 0.0,
            prev + 156
        )?;
        assert_that!(
            "anim def exec by range max",
            anim_def.exec_by_range_max >= anim_def.exec_by_range_min,
            prev + 156
        )?;
        Execution::ByRange(Range {
            min: anim_def.exec_by_range_min,
            max: anim_def.exec_by_range_max,
        })
    } else {
        assert_that!(
            "anim def exec by range min",
            anim_def.exec_by_range_min == 0.0,
            prev + 156
        )?;
        assert_that!(
            "anim def exec by range max",
            anim_def.exec_by_range_max == 0.0,
            prev + 156
        )?;
        if exec_by_zone {
            Execution::ByZone
        } else {
            Execution::None
        }
    };

    let reset_time = if flags.contains(AnimDefFlags::RESET_TIME) {
        Some(anim_def.reset_time)
    } else {
        assert_that!(
            "anim def reset time",
            anim_def.reset_time == -1.0,
            prev + 164
        )?;
        None
    };
    assert_that!("anim def field 168", anim_def.zero168 == 0.0, prev + 168)?;

    assert_that!(
        "anim def max health",
        anim_def.max_health >= 0.0,
        prev + 172
    )?;
    assert_that!(
        "anim def cur health",
        anim_def.cur_health == anim_def.max_health,
        prev + 176
    )?;

    assert_that!("anim def field 180", anim_def.zero180 == 0, prev + 180)?;
    assert_that!("anim def field 184", anim_def.zero184 == 0, prev + 184)?;
    assert_that!("anim def field 188", anim_def.zero188 == 0, prev + 188)?;
    assert_that!("anim def field 192", anim_def.zero192 == 0, prev + 192)?;

    // an anim definition must have at least one sequence definition?
    assert_that!(
        "anim def seq defs pointer",
        anim_def.seq_defs_ptr != 0,
        prev + 196
    )?;

    // reset state
    let (reset_state_ptr, reset_state_size) = anim_def.reset_state.assert_reset_state(prev)?;
    // the reset state pointer and size are used later in `read_reset_state`

    // padding?
    assert_that!("anim def field 275", anim_def.zero275 == 0, prev + 275)?;
    // pointer to next anim_def?
    assert_that!("anim def field 312", anim_def.zero312 == 0, prev + 312)?;

    // an anim definition must have at least one sequence definition?
    assert_that!(
        "anim def seq def count",
        anim_def.seq_def_count > 0,
        prev + 264
    )?;
    // sequence definitions will be read later

    let objects = if anim_def.object_count > 0 {
        assert_that!(
            "anim def objects ptr",
            anim_def.objects_ptr != 0,
            prev + 276
        )?;
        Some(read_objects(read, anim_def.object_count)?)
    } else {
        assert_that!(
            "anim def objects ptr",
            anim_def.objects_ptr == 0,
            prev + 276
        )?;
        None
    };

    let nodes = if anim_def.node_count > 0 {
        assert_that!("anim def nodes ptr", anim_def.nodes_ptr != 0, prev + 280)?;
        Some(read_nodes(read, anim_def.node_count)?)
    } else {
        assert_that!("anim def nodes ptr", anim_def.nodes_ptr == 0, prev + 280)?;
        None
    };

    let lights = if anim_def.light_count > 0 {
        assert_that!("anim def lights ptr", anim_def.lights_ptr != 0, prev + 284)?;
        Some(read_lights(read, anim_def.light_count)?)
    } else {
        assert_that!("anim def lights ptr", anim_def.lights_ptr == 0, prev + 284)?;
        None
    };

    let puffers = if anim_def.puffer_count > 0 {
        assert_that!(
            "anim def puffers ptr",
            anim_def.puffers_ptr != 0,
            prev + 288
        )?;
        Some(read_puffers(read, anim_def.puffer_count)?)
    } else {
        assert_that!(
            "anim def puffers ptr",
            anim_def.puffers_ptr == 0,
            prev + 288
        )?;
        None
    };

    let dynamic_sounds = if anim_def.dynamic_sound_count > 0 {
        assert_that!(
            "anim def dynamic sounds ptr",
            anim_def.dynamic_sounds_ptr != 0,
            prev + 292
        )?;
        Some(read_dynamic_sounds(read, anim_def.dynamic_sound_count)?)
    } else {
        assert_that!(
            "anim def dynamic sounds ptr",
            anim_def.dynamic_sounds_ptr == 0,
            prev + 292
        )?;
        None
    };

    let static_sounds = if anim_def.static_sound_count > 0 {
        assert_that!(
            "anim def static sounds ptr",
            anim_def.static_sounds_ptr != 0,
            prev + 296
        )?;
        Some(read_static_sounds(read, anim_def.static_sound_count)?)
    } else {
        assert_that!(
            "anim def static sounds ptr",
            anim_def.static_sounds_ptr == 0,
            prev + 296
        )?;
        None
    };

    // in MW and PM, effects aren't used (RC only)
    assert_that!(
        "anim def effect count",
        anim_def.effect_count == 0,
        prev + 271
    )?;
    assert_that!(
        "anim def effects ptr",
        anim_def.effects_ptr == 0,
        prev + 300
    )?;

    let activ_prereqs = if anim_def.activ_prereq_count > 0 {
        assert_that!(
            "anim def activ prereqs ptr",
            anim_def.activ_prereqs_ptr != 0,
            prev + 304
        )?;
        assert_that!(
            "anim def activ prereqs min",
            anim_def.activ_prereq_min_to_satisfy in [0, 1, 2],
            prev + 273
        )?;
        Some(read_activ_prereqs(
            read,
            anim_def.activ_prereq_count,
            false,
        )?)
    } else {
        assert_that!(
            "anim def activ prereqs ptr",
            anim_def.activ_prereqs_ptr == 0,
            prev + 304
        )?;
        assert_that!(
            "anim def activ prereqs min",
            anim_def.activ_prereq_min_to_satisfy == 0,
            prev + 273
        )?;
        None
    };

    let anim_refs = if anim_def.anim_ref_count > 0 {
        assert_that!(
            "anim def anim refs ptr",
            anim_def.anim_refs_ptr != 0,
            prev + 308
        )?;
        Some(read_anim_refs(read, anim_def.anim_ref_count)?)
    } else {
        assert_that!(
            "anim def anim refs ptr",
            anim_def.anim_refs_ptr == 0,
            prev + 308
        )?;
        None
    };

    let mut result = AnimDef {
        name,
        anim_name,
        anim_root_name,
        file_name: file_name.clone(),
        has_callbacks: flags.contains(AnimDefFlags::HAS_CALLBACKS),
        auto_reset_node_states: flags.contains(AnimDefFlags::AUTO_RESET_NODE_STATES),
        local_nodes_only: false,
        proximity_damage: flags.contains(AnimDefFlags::PROXIMITY_DAMAGE),
        active: true,
        low_priority: false,
        activation,
        execution,
        network_log,
        save_log,
        reset_time,
        health: anim_def.max_health,
        activ_prereq_min_to_satisfy: anim_def.activ_prereq_min_to_satisfy,
        objects,
        nodes,
        lights,
        puffers,
        dynamic_sounds,
        static_sounds,
        effects: None,
        activ_prereqs,
        anim_refs,
        // these need the anim def to do lookups
        reset_state: None,
        sequences: Vec::new(),
    };

    // unconditional read
    let read_events = ReadEventsMw { scripts };
    result.reset_state = read_reset_state_pg(
        read,
        &result,
        reset_state_size,
        reset_state_ptr,
        read_events,
    )?;

    let read_events = ReadEventsMw { scripts };
    result.sequences = read_sequence_defs(read, &result, anim_def.seq_def_count, read_events)?;

    // TODO
    // // the Callback event checks if callbacks are allowed, but i also wanted to catch
    // // the case where the flag might've been set, but no callbacks exists
    // let mut expect_callbacks = false;
    // for seq in &result.sequences {
    //     for event in &seq.events {
    //         if let EventData::Callback(_) = event.data {
    //             expect_callbacks = true;
    //             break;
    //         }
    //     }
    // }

    // assert_that!(
    //     "anim_def has callbacks",
    //     result.has_callbacks == expect_callbacks,
    //     prev + 148
    // )?;

    let anim_def_names = AnimDefName {
        file_name,
        rename: None,
        anim_ptr: anim_def.anim_ptr,
        anim_root_ptr: anim_def.anim_root_ptr,
        anim_hash,
        anim_root_hash,
        objects_ptr: anim_def.objects_ptr,
        nodes_ptr: anim_def.nodes_ptr,
        lights_ptr: anim_def.lights_ptr,
        puffers_ptr: anim_def.puffers_ptr,
        dynamic_sounds_ptr: anim_def.dynamic_sounds_ptr,
        static_sounds_ptr: anim_def.static_sounds_ptr,
        effects_ptr: 0,
        activ_prereqs_ptr: anim_def.activ_prereqs_ptr,
        anim_refs_ptr: anim_def.anim_refs_ptr,
        seq_defs_ptr: anim_def.seq_defs_ptr,
        reset_state_ptr: 0,
    };
    Ok((result, anim_def_names))
}
