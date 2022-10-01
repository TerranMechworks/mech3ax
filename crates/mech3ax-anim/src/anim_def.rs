use super::activation_prereq::{read_activ_prereqs, write_activ_prereqs};
use super::support::*;
use crate::sequence_event::{read_events, size_events, write_events};
use log::trace;
use mech3ax_api_types::{
    static_assert_size, AnimActivation, AnimDef, AnimPtr, EventData, Execution, NamePad, Range,
    ReprSize as _, ResetState, SeqActivation, SeqDef,
};
use mech3ax_common::assert::{assert_all_zero, assert_utf8, AssertionError};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::string::{
    str_from_c_padded, str_from_c_partition, str_to_c_padded, str_to_c_partition,
};
use mech3ax_common::{assert_that, Result};
use num_traits::FromPrimitive as _;
use std::io::{Read, Write};

bitflags::bitflags! {
    pub struct AnimDefFlags: u32 {
        const EXECUTION_BY_RANGE = 1 << 1;
        const EXECUTION_BY_ZONE = 1 << 3;
        const HAS_CALLBACKS = 1 << 4;
        const RESET_TIME = 1 << 5;
        const NETWORK_LOG_SET = 1 << 10;
        const NETWORK_LOG_ON = 1 << 11;
        const SAVE_LOG_SET = 1 << 12;
        const SAVE_LOG_ON = 1 << 13;
        const AUTO_RESET_NODE_STATES = 1 << 16;
        const PROXIMITY_DAMAGE = 1 << 20;
    }
}

#[repr(C)]
struct AnimDefC {
    anim_name: [u8; 32],             // 000
    name: [u8; 32],                  // 032
    anim_ptr: u32,                   // 064
    anim_root: [u8; 32],             // 068
    anim_root_ptr: u32,              // 100
    zero104: [u8; 44],               // 104
    flags: u32,                      // 148
    status: u8,                      // 152
    activation: u8,                  // 153
    action_prio: u8,                 // 154
    two155: u8,                      // 155
    exec_by_range_min: f32,          // 156
    exec_by_range_max: f32,          // 160
    reset_time: f32,                 // 164
    zero168: f32,                    // 168
    max_health: f32,                 // 172
    cur_health: f32,                 // 176
    zero180: u32,                    // 180
    zero184: u32,                    // 184
    zero188: u32,                    // 188
    zero192: u32,                    // 192
    seq_defs_ptr: u32,               // 196
    reset_state: SeqDefInfoC,        // 200
    seq_def_count: u8,               // 264
    object_count: u8,                // 265
    node_count: u8,                  // 266
    light_count: u8,                 // 267
    puffer_count: u8,                // 268
    dynamic_sound_count: u8,         // 269
    static_sound_count: u8,          // 270
    unknown_count: u8,               // 271
    activ_prereq_count: u8,          // 272
    activ_prereq_min_to_satisfy: u8, // 273
    anim_ref_count: u8,              // 274
    zero275: u8,                     // 275
    objects_ptr: u32,                // 276
    nodes_ptr: u32,                  // 280
    lights_ptr: u32,                 // 284
    puffers_ptr: u32,                // 288
    dynamic_sounds_ptr: u32,         // 292
    static_sounds_ptr: u32,          // 296
    unknown_ptr: u32,                // 300
    activ_prereqs_ptr: u32,          // 304
    anim_refs_ptr: u32,              // 308
    zero312: u32,                    // 312
}
static_assert_size!(AnimDefC, 316);

#[repr(C)]
struct SeqDefInfoC {
    name: [u8; 32],   // 00
    flags: u32,       // 32
    zero36: [u8; 20], // 36
    pointer: u32,     // 56
    size: u32,        // 60
}
static_assert_size!(SeqDefInfoC, 64);
const RESET_SEQUENCE: &[u8; 32] = b"RESET_SEQUENCE\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";

pub fn read_anim_def_zero(read: &mut CountingReader<impl Read>) -> Result<()> {
    // the first entry is always zero
    let mut anim_def = [0; AnimDefC::SIZE as usize];
    read.read_exact(&mut anim_def)?;
    // ...except for this one byte?
    let activation = anim_def[153];
    assert_that!(
        "anim def zero activation",
        activation == AnimActivation::OnCall as u8,
        read.prev + 153
    )?;
    anim_def[153] = 0;
    assert_all_zero("anim def zero header", read.prev, &anim_def)?;
    // reset state
    let mut reset_state = [0; SeqDefInfoC::SIZE as usize];
    read.read_exact(&mut reset_state)?;
    assert_all_zero("anim def zero reset state", read.prev, &reset_state)?;
    Ok(())
}

fn read_reset_state(
    read: &mut CountingReader<impl Read>,
    anim_def: &AnimDef,
    size: u32,
    pointer: u32,
) -> Result<Option<ResetState>> {
    trace!("Reading anim def reset state at {}", read.offset);
    let reset_state: SeqDefInfoC = read.read_struct()?;
    assert_that!(
        "anim def reset state name",
        &reset_state.name == RESET_SEQUENCE,
        read.prev + 0
    )?;
    assert_that!(
        "anim def reset state flags",
        reset_state.flags == 0,
        read.prev + 32
    )?;
    assert_all_zero(
        "anim def reset state field 36",
        read.prev + 36,
        &reset_state.zero36,
    )?;
    assert_that!(
        "anim def reset state pointer",
        reset_state.pointer == pointer,
        read.prev + 56
    )?;
    assert_that!(
        "anim def reset state size",
        reset_state.size == size,
        read.prev + 60
    )?;

    if size > 0 {
        assert_that!(
            "anim def reset state pointer",
            reset_state.pointer != 0,
            read.prev + 56
        )?;
        let events = read_events(read, size, anim_def)?;
        Ok(Some(ResetState { events, pointer }))
    } else {
        assert_that!(
            "anim def reset state pointer",
            reset_state.pointer == 0,
            read.prev + 56
        )?;
        Ok(None)
    }
}

fn read_sequence_def(read: &mut CountingReader<impl Read>, anim_def: &AnimDef) -> Result<SeqDef> {
    let seq_def: SeqDefInfoC = read.read_struct()?;
    let name = assert_utf8("anim def seq def name", read.prev + 0, || {
        str_from_c_padded(&seq_def.name)
    })?;
    let activation = if seq_def.flags == 0 {
        SeqActivation::Initial
    } else if seq_def.flags == 0x0303 {
        SeqActivation::OnCall
    } else {
        let msg = format!(
            "Expected valid seq def flags, but was 0x{:08X} (at {})",
            seq_def.flags,
            read.prev + 32
        );
        return Err(AssertionError(msg).into());
    };
    assert_all_zero("anim def seq def field 36", read.prev + 36, &seq_def.zero36)?;
    // it doesn't make sense for a sequence to be empty
    assert_that!(
        "anim def seq def pointer",
        seq_def.pointer != 0,
        read.prev + 56
    )?;
    assert_that!("anim def seq def size", seq_def.size > 0, read.prev + 60)?;

    let events = read_events(read, seq_def.size, anim_def)?;

    Ok(SeqDef {
        name,
        activation,
        events,
        pointer: seq_def.pointer,
    })
}

fn read_sequence_defs(
    read: &mut CountingReader<impl Read>,
    anim_def: &AnimDef,
    count: u8,
) -> Result<Vec<SeqDef>> {
    (0..count)
        .map(|i| {
            trace!("Reading anim def sequence {} at {}", i, read.offset);
            read_sequence_def(read, anim_def)
        })
        .collect()
}

pub fn read_anim_def(read: &mut CountingReader<impl Read>) -> Result<(AnimDef, AnimPtr)> {
    let anim_def: AnimDefC = read.read_struct()?;

    // save this so we can output accurate offsets after doing further reads
    let prev = read.prev;

    let anim_name = {
        let (name, pad) = assert_utf8("anim def anim name", prev + 0, || {
            str_from_c_partition(&anim_def.anim_name)
        })?;
        NamePad { name, pad }
    };
    let name = assert_utf8("anim def name", prev + 32, || {
        str_from_c_padded(&anim_def.name)
    })?;
    assert_that!("anim def anim ptr", anim_def.anim_ptr != 0, prev + 64)?;
    let anim_root = {
        let (name, pad) = assert_utf8("anim def anim root name", prev + 68, || {
            str_from_c_partition(&anim_def.anim_root)
        })?;
        NamePad { name, pad }
    };

    let base_name = name.replace(".flt", "");
    let file_name = if name != anim_root.name {
        assert_that!(
            "anim def anim root ptr",
            anim_def.anim_root_ptr != anim_def.anim_ptr,
            prev + 100
        )?;
        format!("{}-{}-{}.json", base_name, anim_name.name, anim_root.name)
    } else {
        assert_that!(
            "anim def anim root ptr",
            anim_def.anim_root_ptr == anim_def.anim_ptr,
            prev + 100
        )?;
        format!("{}-{}.json", base_name, anim_name.name)
    };

    assert_all_zero("anim def field 104", prev + 104, &anim_def.zero104)?;

    let flags = AnimDefFlags::from_bits(anim_def.flags).ok_or_else(|| {
        AssertionError(format!(
            "Expected valid anim def flags, but was 0x{:04X} (at {})",
            anim_def.flags,
            read.prev + 148
        ))
    })?;

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
    let activation = AnimActivation::from_u8(anim_def.activation).ok_or_else(|| {
        AssertionError(format!(
            "Expected valid anim def activation, but was {} (at {})",
            anim_def.activation,
            read.prev + 153
        ))
    })?;
    assert_that!(
        "anim def action priority",
        anim_def.action_prio == 4,
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

    // reset state
    assert_that!(
        "anim def reset state name",
        &anim_def.reset_state.name == RESET_SEQUENCE,
        prev + 200
    )?;
    assert_that!(
        "anim def reset state flags",
        anim_def.reset_state.flags == 0,
        prev + 232
    )?;
    assert_all_zero(
        "anim def reset state field 36",
        prev + 236,
        &anim_def.reset_state.zero36,
    )?;
    // the reset state pointer and size are used later in `read_reset_state`

    // padding?
    assert_that!("anim def field 275", anim_def.zero275 == 0, prev + 275)?;
    // pointer to next anim_def?
    assert_that!("anim def field 312", anim_def.zero312 == 0, prev + 312)?;

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

    // this isn't set in any file i have
    assert_that!(
        "anim def field 271",
        anim_def.unknown_count == 0,
        prev + 271
    )?;
    assert_that!("anim def field 300", anim_def.unknown_ptr == 0, prev + 300)?;

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
        Some(read_activ_prereqs(read, anim_def.activ_prereq_count)?)
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
        anim_root,
        file_name: file_name.clone(),
        auto_reset_node_states: flags.contains(AnimDefFlags::AUTO_RESET_NODE_STATES),
        activation,
        execution,
        network_log,
        save_log,
        has_callbacks: flags.contains(AnimDefFlags::HAS_CALLBACKS),
        reset_time,
        health: anim_def.max_health,
        proximity_damage: flags.contains(AnimDefFlags::PROXIMITY_DAMAGE),
        activ_prereq_min_to_satisfy: anim_def.activ_prereq_min_to_satisfy,
        objects,
        nodes,
        lights,
        puffers,
        dynamic_sounds,
        static_sounds,
        activ_prereqs,
        anim_refs,
        // these need the anim def to do lookups
        reset_state: None,
        sequences: Vec::new(),
    };

    // unconditional read
    result.reset_state = read_reset_state(
        read,
        &result,
        anim_def.reset_state.size,
        anim_def.reset_state.pointer,
    )?;

    // this could be zero, in which case the pointer would also be NULL? (but never is)
    assert_that!(
        "anim def seq def count",
        anim_def.seq_def_count > 0,
        prev + 264
    )?;
    assert_that!(
        "anim def seq defs pointer",
        anim_def.seq_defs_ptr != 0,
        prev + 196
    )?;

    result.sequences = read_sequence_defs(read, &result, anim_def.seq_def_count)?;

    // the Callback event checks if callbacks are allowed, but i also wanted to catch
    // the case where the flag might've been set, but no callbacks exists
    let mut expect_callbacks = false;
    for seq in &result.sequences {
        for event in &seq.events {
            if let EventData::Callback(_) = event.data {
                expect_callbacks = true;
                break;
            }
        }
    }

    assert_that!(
        "anim_def has callbacks",
        result.has_callbacks == expect_callbacks,
        prev + 148
    )?;

    let anim_ptr = AnimPtr {
        file_name,
        anim_ptr: anim_def.anim_ptr,
        anim_root_ptr: anim_def.anim_root_ptr,
        objects_ptr: anim_def.objects_ptr,
        nodes_ptr: anim_def.nodes_ptr,
        lights_ptr: anim_def.lights_ptr,
        puffers_ptr: anim_def.puffers_ptr,
        dynamic_sounds_ptr: anim_def.dynamic_sounds_ptr,
        static_sounds_ptr: anim_def.static_sounds_ptr,
        activ_prereqs_ptr: anim_def.activ_prereqs_ptr,
        anim_refs_ptr: anim_def.anim_refs_ptr,
        reset_state_ptr: anim_def.reset_state.pointer,
        seq_defs_ptr: anim_def.seq_defs_ptr,
    };
    Ok((result, anim_ptr))
}

pub fn write_anim_def_zero(write: &mut CountingWriter<impl Write>) -> Result<()> {
    // the first entry is always zero
    let mut anim_def = [0; AnimDefC::SIZE as usize];
    // ...except for this one byte?
    anim_def[153] = AnimActivation::OnCall as u8;
    write.write_all(&anim_def)?;
    // reset state
    write.write_zeros(SeqDefInfoC::SIZE)?;
    Ok(())
}

fn write_reset_state(
    write: &mut CountingWriter<impl Write>,
    anim_def: &AnimDef,
    size: u32,
) -> Result<()> {
    write.write_struct(&SeqDefInfoC {
        name: RESET_SEQUENCE.clone(),
        flags: 0,
        zero36: [0; 20],
        pointer: anim_def
            .reset_state
            .as_ref()
            .map(|state| state.pointer)
            .unwrap_or(0),
        size,
    })?;

    if let Some(state) = &anim_def.reset_state {
        write_events(write, anim_def, &state.events)?;
    }
    Ok(())
}

fn write_sequence_defs(write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
    for seq_def in &anim_def.sequences {
        let mut name = [0; 32];
        str_to_c_padded(&seq_def.name, &mut name);
        let flags = if seq_def.activation == SeqActivation::OnCall {
            0x0303
        } else {
            0
        };
        let size = size_events(&seq_def.events);
        write.write_struct(&SeqDefInfoC {
            name,
            flags,
            zero36: [0; 20],
            pointer: seq_def.pointer,
            size,
        })?;
        write_events(write, anim_def, &seq_def.events)?;
    }
    Ok(())
}

pub fn write_anim_def(
    write: &mut CountingWriter<impl Write>,
    anim_def: &AnimDef,
    anim_ptr: &AnimPtr,
) -> Result<()> {
    let mut anim_name = [0; 32];
    str_to_c_partition(
        &anim_def.anim_name.name,
        &anim_def.anim_name.pad,
        &mut anim_name,
    );
    let mut name = [0; 32];
    str_to_c_padded(&anim_def.name, &mut name);
    let mut anim_root = [0; 32];
    str_to_c_partition(
        &anim_def.anim_root.name,
        &anim_def.anim_root.pad,
        &mut anim_root,
    );

    let mut flags = AnimDefFlags::empty();
    if let Some(network_log_on) = &anim_def.network_log {
        flags |= AnimDefFlags::NETWORK_LOG_SET;
        if *network_log_on {
            flags |= AnimDefFlags::NETWORK_LOG_ON;
        }
    }
    if let Some(save_log_on) = &anim_def.save_log {
        flags |= AnimDefFlags::SAVE_LOG_SET;
        if *save_log_on {
            flags |= AnimDefFlags::SAVE_LOG_ON;
        }
    }

    let (exec_by_range_min, exec_by_range_max) = match &anim_def.execution {
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
    if anim_def.proximity_damage {
        flags |= AnimDefFlags::PROXIMITY_DAMAGE;
    }

    let object_count = anim_def.objects.as_ref().map(|v| v.len() + 1).unwrap_or(0) as u8;
    let node_count = anim_def.nodes.as_ref().map(|v| v.len() + 1).unwrap_or(0) as u8;
    let light_count = anim_def.lights.as_ref().map(|v| v.len() + 1).unwrap_or(0) as u8;
    let puffer_count = anim_def.puffers.as_ref().map(|v| v.len() + 1).unwrap_or(0) as u8;
    let dynamic_sound_count = anim_def
        .dynamic_sounds
        .as_ref()
        .map(|v| v.len() + 1)
        .unwrap_or(0) as u8;
    let static_sound_count = anim_def
        .static_sounds
        .as_ref()
        .map(|v| v.len() + 1)
        .unwrap_or(0) as u8;
    let activ_prereq_count = anim_def
        .activ_prereqs
        .as_ref()
        .map(|v| v.len())
        .unwrap_or(0) as u8;
    let anim_ref_count = anim_def.anim_refs.as_ref().map(|v| v.len()).unwrap_or(0) as u8;

    let reset_state_events_size = anim_def
        .reset_state
        .as_ref()
        .map(|state| size_events(&state.events))
        .unwrap_or(0);

    let reset_state = SeqDefInfoC {
        name: RESET_SEQUENCE.clone(),
        flags: 0,
        zero36: [0; 20],
        pointer: anim_ptr.reset_state_ptr,
        size: reset_state_events_size,
    };

    write.write_struct(&AnimDefC {
        anim_name,
        name,
        anim_ptr: anim_ptr.anim_ptr,
        anim_root,
        anim_root_ptr: anim_ptr.anim_root_ptr,
        zero104: [0; 44],
        flags: flags.bits(),
        status: 0,
        activation: anim_def.activation as u8,
        action_prio: 4,
        two155: 2,
        exec_by_range_min,
        exec_by_range_max,
        reset_time: anim_def.reset_time.unwrap_or(-1.0),
        zero168: 0.0,
        max_health: anim_def.health,
        cur_health: anim_def.health,
        zero180: 0,
        zero184: 0,
        zero188: 0,
        zero192: 0,
        seq_defs_ptr: anim_ptr.seq_defs_ptr,
        reset_state,
        seq_def_count: anim_def.sequences.len() as u8,
        object_count,
        node_count,
        light_count,
        puffer_count,
        dynamic_sound_count,
        static_sound_count,
        unknown_count: 0,
        activ_prereq_count,
        activ_prereq_min_to_satisfy: anim_def.activ_prereq_min_to_satisfy,
        anim_ref_count,
        zero275: 0,
        objects_ptr: anim_ptr.objects_ptr,
        nodes_ptr: anim_ptr.nodes_ptr,
        lights_ptr: anim_ptr.lights_ptr,
        puffers_ptr: anim_ptr.puffers_ptr,
        dynamic_sounds_ptr: anim_ptr.dynamic_sounds_ptr,
        static_sounds_ptr: anim_ptr.static_sounds_ptr,
        unknown_ptr: 0,
        activ_prereqs_ptr: anim_ptr.activ_prereqs_ptr,
        anim_refs_ptr: anim_ptr.anim_refs_ptr,
        zero312: 0,
    })?;

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
        write_activ_prereqs(write, activ_prereqs)?;
    }
    if let Some(anim_refs) = &anim_def.anim_refs {
        write_anim_refs(write, anim_refs)?;
    }
    write_reset_state(write, anim_def, reset_state_events_size)?;
    write_sequence_defs(write, anim_def)?;

    Ok(())
}
