use super::{AnimDefC, Flags};
use mech3ax_api_types::anim::AnimActivation;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use std::io::{Read, Write};

pub(crate) fn write_anim_def_zero(write: &mut CountingWriter<impl Write>) -> Result<()> {
    // --- anim def

    // the first entry is always zero...
    let anim_def = AnimDefC {
        // ... except for this...
        anim_ptr: u32::MAX,
        // ... except for this...
        anim_root_ptr: u32::MAX,
        // ... and except for this
        activation: AnimActivation::OnCall.maybe(),
        ..Default::default()
    };
    write.write_struct(&anim_def)?;

    Ok(())
}

pub(crate) fn read_anim_def_zero(read: &mut CountingReader<impl Read>) -> Result<()> {
    let anim_def: AnimDefC = read.read_struct()?;

    // the first entry is always zero...
    assert_that!(
        "anim def zero field 000",
        zero anim_def.anim_name,
        read.prev + 0
    )?;
    assert_that!(
        "anim def zero field 032",
        anim_def.unknowns_ptr == 0,
        read.prev + 32
    )?;
    assert_that!(
        "anim def zero field 036",
        anim_def.unknowns_count == 0,
        read.prev + 36
    )?;
    assert_that!(
        "anim def zero field 040",
        zero anim_def.name,
        read.prev + 40
    )?;
    // ... except for this...
    assert_that!(
        "anim def zero field 072",
        anim_def.anim_ptr == u32::MAX,
        read.prev + 72
    )?;
    assert_that!(
        "anim def zero field 076",
        zero anim_def.anim_root_name,
        read.prev + 76
    )?;
    // ... except for this...
    assert_that!(
        "anim def zero field 108",
        anim_def.anim_root_ptr == u32::MAX,
        read.prev + 108
    )?;
    assert_that!(
        "anim def zero field 112",
        zero anim_def.zero112,
        read.prev + 112
    )?;
    assert_that!(
        "anim def zero field 156",
        anim_def.flags == Flags::empty(),
        read.prev + 156
    )?;
    assert_that!(
        "anim def zero field 160",
        anim_def.status == 0,
        read.prev + 160
    )?;
    // ... and except for this.
    assert_that!(
        "anim def zero field 161",
        anim_def.activation == AnimActivation::OnCall.maybe(),
        read.prev + 161
    )?;
    assert_that!(
        "anim def zero field 162",
        anim_def.execution_priority == 0,
        read.prev + 162
    )?;
    assert_that!(
        "anim def zero field 163",
        anim_def.two163 == 0,
        read.prev + 163
    )?;
    assert_that!(
        "anim def zero field 164",
        anim_def.exec_by_range_min == 0.0,
        read.prev + 164
    )?;
    assert_that!(
        "anim def zero field 168",
        anim_def.exec_by_range_max == 0.0,
        read.prev + 168
    )?;
    assert_that!(
        "anim def zero field 172",
        anim_def.reset_time == 0.0,
        read.prev + 172
    )?;
    assert_that!(
        "anim def zero field 176",
        anim_def.zero176 == 0.0,
        read.prev + 176
    )?;
    assert_that!(
        "anim def zero field 180",
        anim_def.max_health == 0.0,
        read.prev + 180
    )?;
    assert_that!(
        "anim def zero field 184",
        anim_def.cur_health == 0.0,
        read.prev + 184
    )?;
    assert_that!(
        "anim def zero field 188",
        anim_def.zero188 == 0,
        read.prev + 188
    )?;
    assert_that!(
        "anim def zero field 192",
        anim_def.zero192 == 0,
        read.prev + 192
    )?;
    assert_that!(
        "anim def zero field 196",
        anim_def.zero196 == 0,
        read.prev + 196
    )?;
    assert_that!(
        "anim def zero field 200",
        anim_def.zero200 == 0,
        read.prev + 200
    )?;
    assert_that!(
        "anim def zero field 204",
        anim_def.seq_defs_ptr == 0,
        read.prev + 204
    )?;
    assert_that!(
        "anim def zero field 208",
        anim_def.reset_state_ptr == 0,
        read.prev + 208
    )?;
    assert_that!(
        "anim def zero field 212",
        anim_def.unknown_seq_ptr == 0,
        read.prev + 212
    )?;
    assert_that!(
        "anim def zero field 216",
        anim_def.seq_def_count == 0,
        read.prev + 216
    )?;
    assert_that!(
        "anim def zero field 217",
        anim_def.object_count == 0,
        read.prev + 217
    )?;
    assert_that!(
        "anim def zero field 218",
        anim_def.node_count == 0,
        read.prev + 218
    )?;
    assert_that!(
        "anim def zero field 219",
        anim_def.light_count == 0,
        read.prev + 219
    )?;
    assert_that!(
        "anim def zero field 220",
        anim_def.puffer_count == 0,
        read.prev + 220
    )?;
    assert_that!(
        "anim def zero field 221",
        anim_def.dynamic_sound_count == 0,
        read.prev + 221
    )?;
    assert_that!(
        "anim def zero field 222",
        anim_def.static_sound_count == 0,
        read.prev + 222
    )?;
    assert_that!(
        "anim def zero field 223",
        anim_def.effect_count == 0,
        read.prev + 223
    )?;
    assert_that!(
        "anim def zero field 224",
        anim_def.activ_prereq_count == 0,
        read.prev + 224
    )?;
    assert_that!(
        "anim def zero field 225",
        anim_def.activ_prereq_min_to_satisfy == 0,
        read.prev + 225
    )?;
    assert_that!(
        "anim def zero field 226",
        anim_def.anim_ref_count == 0,
        read.prev + 226
    )?;
    assert_that!(
        "anim def zero field 227",
        anim_def.zero227 == 0,
        read.prev + 227
    )?;
    assert_that!(
        "anim def zero field 228",
        anim_def.objects_ptr == 0,
        read.prev + 228
    )?;
    assert_that!(
        "anim def zero field 232",
        anim_def.nodes_ptr == 0,
        read.prev + 232
    )?;
    assert_that!(
        "anim def zero field 236",
        anim_def.lights_ptr == 0,
        read.prev + 236
    )?;
    assert_that!(
        "anim def zero field 240",
        anim_def.puffers_ptr == 0,
        read.prev + 240
    )?;
    assert_that!(
        "anim def zero field 244",
        anim_def.dynamic_sounds_ptr == 0,
        read.prev + 244
    )?;
    assert_that!(
        "anim def zero field 248",
        anim_def.static_sounds_ptr == 0,
        read.prev + 248
    )?;
    assert_that!(
        "anim def zero field 252",
        anim_def.effects_ptr == 0,
        read.prev + 252
    )?;
    assert_that!(
        "anim def zero field 256",
        anim_def.activ_prereqs_ptr == 0,
        read.prev + 256
    )?;
    assert_that!(
        "anim def zero field 260",
        anim_def.anim_refs_ptr == 0,
        read.prev + 260
    )?;
    assert_that!(
        "anim def zero field 264",
        anim_def.zero264 == 0,
        read.prev + 264
    )?;

    Ok(())
}
