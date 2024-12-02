use super::{AnimDefC, Flags};
use crate::common::seq_def::SeqDefInfoC;
use mech3ax_api_types::anim::AnimActivation;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use std::io::{Read, Write};

pub(crate) fn write_anim_def_zero(write: &mut CountingWriter<impl Write>) -> Result<()> {
    // --- anim def

    // the first entry is always zero...
    let anim_def = AnimDefC {
        // ...except for this.
        activation: AnimActivation::OnCall.maybe(),
        ..Default::default()
    };
    write.write_struct(&anim_def)?;

    // --- reset state
    let reset_state = SeqDefInfoC::default();
    write.write_struct(&reset_state)?;

    Ok(())
}

pub(crate) fn read_anim_def_zero(read: &mut CountingReader<impl Read>) -> Result<()> {
    // --- anim def
    let anim_def: AnimDefC = read.read_struct()?;

    // the first entry is always zero...
    assert_that!(
        "anim def zero field 000",
        zero anim_def.anim_name,
        read.prev + 0
    )?;
    assert_that!(
        "anim def zero field 032",
        zero anim_def.name,
        read.prev + 32
    )?;
    assert_that!(
        "anim def zero field 064",
        anim_def.anim_ptr == 0,
        read.prev + 64
    )?;
    assert_that!(
        "anim def zero field 068",
        zero anim_def.anim_root_name,
        read.prev + 68
    )?;
    assert_that!(
        "anim def zero field 100",
        anim_def.anim_root_ptr == 0,
        read.prev + 100
    )?;
    assert_that!(
        "anim def zero field 104",
        zero anim_def.zero104,
        read.prev + 104
    )?;
    assert_that!(
        "anim def zero field 148",
        anim_def.flags == Flags::empty(),
        read.prev + 148
    )?;
    assert_that!(
        "anim def zero field 152",
        anim_def.status == 0,
        read.prev + 152
    )?;
    // ...except for this.
    assert_that!(
        "anim def zero field 153",
        anim_def.activation == AnimActivation::OnCall.maybe(),
        read.prev + 153
    )?;
    assert_that!(
        "anim def zero field 154",
        anim_def.execution_priority == 0,
        read.prev + 154
    )?;
    assert_that!(
        "anim def zero field 155",
        anim_def.two155 == 0,
        read.prev + 155
    )?;
    assert_that!(
        "anim def zero field 156",
        anim_def.exec_by_range_min == 0.0,
        read.prev + 156
    )?;
    assert_that!(
        "anim def zero field 160",
        anim_def.exec_by_range_max == 0.0,
        read.prev + 160
    )?;
    assert_that!(
        "anim def zero field 164",
        anim_def.reset_time == 0.0,
        read.prev + 164
    )?;
    assert_that!(
        "anim def zero field 168",
        anim_def.zero168 == 0.0,
        read.prev + 168
    )?;
    assert_that!(
        "anim def zero field 172",
        anim_def.max_health == 0.0,
        read.prev + 172
    )?;
    assert_that!(
        "anim def zero field 176",
        anim_def.cur_health == 0.0,
        read.prev + 176
    )?;
    assert_that!(
        "anim def zero field 180",
        anim_def.zero180 == 0,
        read.prev + 180
    )?;
    assert_that!(
        "anim def zero field 184",
        anim_def.zero184 == 0,
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
        anim_def.seq_defs_ptr == 0,
        read.prev + 196
    )?;

    assert_that!(
        "anim def zero field 200",
        zero anim_def.reset_state.name,
        read.prev + 200
    )?;
    assert_that!(
        "anim def zero field 232",
        anim_def.reset_state.flags == 0,
        read.prev + 232
    )?;
    assert_that!(
        "anim def zero field 236",
        zero anim_def.reset_state.zero36,
        read.prev + 236
    )?;
    assert_that!(
        "anim def zero field 256",
        anim_def.reset_state.pointer == 0,
        read.prev + 256
    )?;
    assert_that!(
        "anim def zero field 260",
        anim_def.reset_state.size == 0,
        read.prev + 260
    )?;

    assert_that!(
        "anim def zero field 264",
        anim_def.seq_def_count == 0,
        read.prev + 264
    )?;
    assert_that!(
        "anim def zero field 265",
        anim_def.object_count == 0,
        read.prev + 265
    )?;
    assert_that!(
        "anim def zero field 266",
        anim_def.node_count == 0,
        read.prev + 266
    )?;
    assert_that!(
        "anim def zero field 267",
        anim_def.light_count == 0,
        read.prev + 267
    )?;
    assert_that!(
        "anim def zero field 268",
        anim_def.puffer_count == 0,
        read.prev + 268
    )?;
    assert_that!(
        "anim def zero field 269",
        anim_def.dynamic_sound_count == 0,
        read.prev + 269
    )?;
    assert_that!(
        "anim def zero field 270",
        anim_def.static_sound_count == 0,
        read.prev + 270
    )?;
    assert_that!(
        "anim def zero field 271",
        anim_def.effect_count == 0,
        read.prev + 271
    )?;
    assert_that!(
        "anim def zero field 272",
        anim_def.activ_prereq_count == 0,
        read.prev + 272
    )?;
    assert_that!(
        "anim def zero field 273",
        anim_def.activ_prereq_min_to_satisfy == 0,
        read.prev + 273
    )?;
    assert_that!(
        "anim def zero field 274",
        anim_def.anim_ref_count == 0,
        read.prev + 274
    )?;
    assert_that!(
        "anim def zero field 275",
        anim_def.zero275 == 0,
        read.prev + 275
    )?;
    assert_that!(
        "anim def zero field 276",
        anim_def.objects_ptr == 0,
        read.prev + 276
    )?;
    assert_that!(
        "anim def zero field 280",
        anim_def.nodes_ptr == 0,
        read.prev + 280
    )?;
    assert_that!(
        "anim def zero field 284",
        anim_def.lights_ptr == 0,
        read.prev + 284
    )?;
    assert_that!(
        "anim def zero field 288",
        anim_def.puffers_ptr == 0,
        read.prev + 288
    )?;
    assert_that!(
        "anim def zero field 292",
        anim_def.dynamic_sounds_ptr == 0,
        read.prev + 292
    )?;
    assert_that!(
        "anim def zero field 296",
        anim_def.static_sounds_ptr == 0,
        read.prev + 296
    )?;
    assert_that!(
        "anim def zero field 300",
        anim_def.effects_ptr == 0,
        read.prev + 300
    )?;
    assert_that!(
        "anim def zero field 304",
        anim_def.activ_prereqs_ptr == 0,
        read.prev + 304
    )?;
    assert_that!(
        "anim def zero field 308",
        anim_def.anim_refs_ptr == 0,
        read.prev + 308
    )?;
    assert_that!(
        "anim def zero field 312",
        anim_def.zero312 == 0,
        read.prev + 312
    )?;

    // --- reset state
    let reset_state: SeqDefInfoC = read.read_struct()?;

    assert_that!("reset state zero field 00", zero reset_state.name, read.prev + 0)?;
    assert_that!(
        "reset state zero field 32",
        reset_state.flags == 0,
        read.prev + 32
    )?;
    assert_that!("reset state zero field 36", zero reset_state.zero36, read.prev + 36)?;
    assert_that!(
        "reset state zero field 56",
        reset_state.pointer == 0,
        read.prev + 56
    )?;
    assert_that!(
        "reset state zero field 60",
        reset_state.size == 0,
        read.prev + 60
    )?;

    Ok(())
}
