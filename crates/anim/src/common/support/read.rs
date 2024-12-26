use super::{
    AnimRefC, AnimRefType, DynamicSoundRefC, EffectRefC, LightRefC, NodeRefC, ObjectRefC,
    PufferRefC, StaticSoundRefC, ABORT_TEST_RAW, ABORT_TEST_STR,
};
use log::{debug, trace};
use mech3ax_api_types::anim::{
    AnimRef, AnimRefCallAnimation, AnimRefCallObjectConnector, DynamicSoundRef, EffectRef,
    LightRef, NodeRef, ObjectRef, PufferRef, StaticSoundRef,
};
use mech3ax_api_types::AffineMatrix;
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{assert_that, Result};
use mech3ax_types::Ptr;
use std::collections::HashSet;
use std::io::Read;

pub(crate) fn affine_to_bin(affine: &AffineMatrix) -> Vec<u8> {
    let bytes: &[u8; 48] = bytemuck::must_cast_ref(affine);
    bytes.to_vec()
}

pub(crate) fn read_objects(
    read: &mut CountingReader<impl Read>,
    count: u8,
) -> Result<Vec<ObjectRef>> {
    trace!("Reading anim def object 0");
    // the first entry is always zero
    let object_c: ObjectRefC = read.read_struct()?;

    assert_that!("anim def object zero name", zero object_c.name, read.prev + 0)?;
    assert_that!(
        "anim def object zero field 32",
        object_c.zero32 == 0,
        read.prev + 32
    )?;
    assert_that!(
        "anim def object zero ptr",
        object_c.ptr == Ptr::NULL,
        read.prev + 36
    )?;
    assert_that!(
        "anim def object zero flags",
        object_c.flags == 0,
        read.prev + 40
    )?;
    assert_that!(
        "anim def object zero flags merged",
        object_c.flags_merged == 0,
        read.prev + 44
    )?;
    assert_that!(
        "anim def object zero affine",
        object_c.affine == AffineMatrix::ZERO,
        read.prev + 48
    )?;

    let mut seen = HashSet::with_capacity(usize::from(count));
    (1..count)
        .map(|index| {
            trace!("Reading anim def object {}", index);
            let object_c: ObjectRefC = read.read_struct()?;

            let name = if object_c.name == ABORT_TEST_RAW {
                debug!("anim def object name `abort_test` fixup");
                ABORT_TEST_STR.to_string()
            } else {
                assert_utf8("anim def object name", read.prev + 0, || {
                    object_c.name.to_str_node_name()
                })?
            };
            if !seen.insert(name.clone()) {
                log::error!("anim def duplicate object ref `{}`", name);
            }

            assert_that!(
                "anim def object field 32",
                object_c.zero32 == 0,
                read.prev + 32
            )?;
            let affine = affine_to_bin(&object_c.affine);

            Ok(ObjectRef {
                name,
                ptr: object_c.ptr.0,
                flags: object_c.flags.0,
                flags_merged: object_c.flags_merged.0,
                affine,
            })
        })
        .collect()
}

pub(crate) fn read_nodes(read: &mut CountingReader<impl Read>, count: u8) -> Result<Vec<NodeRef>> {
    trace!("Reading anim def node 0");
    // the first entry is always zero
    let node_c: NodeRefC = read.read_struct()?;

    assert_that!("anim def node zero name", zero node_c.name, read.prev + 0)?;
    assert_that!(
        "anim def node zero field 32",
        node_c.zero32 == 0,
        read.prev + 32
    )?;
    assert_that!(
        "anim def node zero pointer",
        node_c.ptr == Ptr::NULL,
        read.prev + 36
    )?;

    let mut seen = HashSet::with_capacity(usize::from(count));
    (1..count)
        .map(|index| {
            trace!("Reading anim def node {}", index);
            let node_c: NodeRefC = read.read_struct()?;

            let name = if node_c.name == ABORT_TEST_RAW {
                debug!("anim def node name `abort_test` fixup");
                ABORT_TEST_STR.to_string()
            } else {
                assert_utf8("anim def node name", read.prev + 0, || {
                    node_c.name.to_str_node_name()
                })?
            };
            if !seen.insert(name.clone()) {
                log::error!("anim def duplicate node ref `{}`", name);
            }

            assert_that!("anim def node field 32", node_c.zero32 == 0, read.prev + 32)?;
            assert_that!(
                "anim def node pointer",
                node_c.ptr != Ptr::NULL,
                read.prev + 36
            )?;
            Ok(NodeRef {
                name,
                ptr: node_c.ptr.into(),
            })
        })
        .collect()
}

pub(crate) fn read_lights(
    read: &mut CountingReader<impl Read>,
    count: u8,
) -> Result<Vec<LightRef>> {
    trace!("Reading anim def light 0");
    // the first entry is always zero
    let light_c: LightRefC = read.read_struct()?;

    assert_that!("anim def light zero name", zero light_c.name, read.prev + 0)?;
    assert_that!(
        "anim def light zero flags",
        light_c.flags == 0,
        read.prev + 32
    )?;
    assert_that!(
        "anim def light zero pointer",
        light_c.ptr == Ptr::NULL,
        read.prev + 36
    )?;
    assert_that!(
        "anim def light zero in world",
        light_c.in_world == 0,
        read.prev + 40
    )?;

    let mut seen = HashSet::with_capacity(usize::from(count));
    (1..count)
        .map(|index| {
            trace!("Reading anim def light {}", index);
            let light_c: LightRefC = read.read_struct()?;

            let name = assert_utf8("anim def light name", read.prev + 0, || {
                light_c.name.to_str_node_name()
            })?;
            if !seen.insert(name.clone()) {
                log::error!("anim def duplicate light ref `{}`", name);
            }

            assert_that!("anim def light flags", light_c.flags == 0, read.prev + 32)?;
            assert_that!(
                "anim def light pointer",
                light_c.ptr != Ptr::NULL,
                read.prev + 36
            )?;
            // if this were non-zero, it would cause the light to be removed instead of added (???)
            assert_that!(
                "anim def light in world",
                light_c.in_world == 0,
                read.prev + 40
            )?;
            Ok(LightRef {
                name,
                ptr: light_c.ptr.into(),
            })
        })
        .collect()
}

pub(crate) fn read_puffers(
    read: &mut CountingReader<impl Read>,
    count: u8,
) -> Result<Vec<PufferRef>> {
    trace!("Reading anim def puffer 0");
    // the first entry is always zero
    let puffer_c: PufferRefC = read.read_struct()?;

    assert_that!("anim def puffer zero name", zero puffer_c.name, read.prev + 0)?;
    assert_that!(
        "anim def puffer zero flags",
        puffer_c.flags == 0,
        read.prev + 32
    )?;
    assert_that!(
        "anim def puffer zero pointer",
        puffer_c.ptr == Ptr::NULL,
        read.prev + 36
    )?;
    assert_that!(
        "anim def puffer zero in world",
        puffer_c.in_world == 0,
        read.prev + 40
    )?;

    let mut seen = HashSet::with_capacity(usize::from(count));
    (1..count)
        .map(|index| {
            trace!("Reading anim def puffer {}", index);
            let puffer_c: PufferRefC = read.read_struct()?;

            let name = assert_utf8("anim def puffer name", read.prev + 0, || {
                puffer_c.name.to_str_padded()
            })?;
            if !seen.insert(name.clone()) {
                log::error!("anim def duplicate puffer ref `{}`", name);
            }

            // TODO: what does this flag mean?
            let flags = puffer_c.flags & 0x00FFFFFF;
            assert_that!("anim def node puffer flags", flags == 0, read.prev + 32)?;
            let flags = (puffer_c.flags >> 24) as u8;

            assert_that!(
                "anim def puffer pointer",
                puffer_c.ptr != Ptr::NULL,
                read.prev + 36
            )?;
            assert_that!(
                "anim def puffer in world",
                puffer_c.in_world == 0,
                read.prev + 40
            )?;
            Ok(PufferRef {
                name,
                flags,
                ptr: puffer_c.ptr.into(),
            })
        })
        .collect()
}

pub(crate) fn read_dynamic_sounds(
    read: &mut CountingReader<impl Read>,
    count: u8,
) -> Result<Vec<DynamicSoundRef>> {
    trace!("Reading anim def dyn sound 0");
    // the first entry is always zero
    let sound_c: DynamicSoundRefC = read.read_struct()?;

    assert_that!("anim def dyn sound zero name", zero sound_c.name, read.prev + 0)?;
    assert_that!(
        "anim def dyn sound zero flags",
        sound_c.flags == 0,
        read.prev + 32
    )?;
    assert_that!(
        "anim def dyn sound zero pointer",
        sound_c.ptr == Ptr::NULL,
        read.prev + 36
    )?;
    assert_that!(
        "anim def dyn sound zero in world",
        sound_c.in_world == 0,
        read.prev + 40
    )?;

    let mut seen = HashSet::with_capacity(usize::from(count));
    (1..count)
        .map(|index| {
            trace!("Reading anim def dyn sound {}", index);
            let sound_c: DynamicSoundRefC = read.read_struct()?;

            let name = assert_utf8("anim def dyn sound name", read.prev + 0, || {
                sound_c.name.to_str_node_name()
            })?;
            if !seen.insert(name.clone()) {
                log::error!("anim def duplicate dyn sound ref `{}`", name);
            }

            assert_that!(
                "anim def dyn sound flags",
                sound_c.flags == 0,
                read.prev + 32
            )?;
            assert_that!(
                "anim def dyn sound pointer",
                sound_c.ptr != Ptr::NULL,
                read.prev + 36
            )?;
            assert_that!(
                "anim def dyn sound in world",
                sound_c.flags == 0,
                read.prev + 40
            )?;
            Ok(DynamicSoundRef {
                name,
                ptr: sound_c.ptr.into(),
            })
        })
        .collect()
}

pub(crate) fn read_static_sounds(
    read: &mut CountingReader<impl Read>,
    count: u8,
) -> Result<Vec<StaticSoundRef>> {
    trace!("Reading anim def stc sound 0");
    // the first entry is always zero
    let sound_c: StaticSoundRefC = read.read_struct()?;

    assert_that!("anim def stc sound zero name", zero sound_c.name, read.prev + 0)?;
    assert_that!(
        "anim def stc sound zero field 32",
        sound_c.zero32 == 0,
        read.prev + 32
    )?;

    let mut seen = HashSet::with_capacity(usize::from(count));
    (1..count)
        .map(|index| {
            trace!("Reading anim def stc sound {}", index);
            let sound_c: StaticSoundRefC = read.read_struct()?;

            // these are full of garbage
            let (name, pad) = assert_utf8("anim def stc sound name", read.prev + 0, || {
                sound_c.name.to_str_garbage()
            })?;
            if !seen.insert(name.clone()) {
                log::error!("anim def duplicate stc sound ref `{}`", name);
            }

            assert_that!(
                "anim def stc sound field 32",
                sound_c.zero32 == 0,
                read.prev + 32
            )?;
            Ok(StaticSoundRef { name, pad })
        })
        .collect()
}

pub(crate) fn read_effects(
    read: &mut CountingReader<impl Read>,
    count: u8,
) -> Result<Vec<EffectRef>> {
    trace!("Reading anim def effect 0");
    // the first entry is always zero
    let effect_c: EffectRefC = read.read_struct()?;

    assert_that!("anim def effect zero name", zero effect_c.name, read.prev + 0)?;
    assert_that!(
        "anim def effect zero field 32",
        effect_c.unk32 == 0,
        read.prev + 32
    )?;

    let mut seen = HashSet::with_capacity(usize::from(count));
    (1..count)
        .map(|index| {
            trace!("Reading anim def effect {}", index);
            let effect_c: EffectRefC = read.read_struct()?;

            // these are full of garbage
            let (name, pad) = assert_utf8("anim def effect name", read.prev + 0, || {
                effect_c.name.to_str_garbage()
            })?;
            if !seen.insert(name.clone()) {
                log::error!("anim def duplicate effect ref `{}`", name);
            }

            assert_that!(
                "anim def effect field 32",
                effect_c.unk32 in (0..12),
                read.prev + 32
            )?;
            Ok(EffectRef {
                name,
                pad,
                unk32: effect_c.unk32,
            })
        })
        .collect()
}

pub(crate) fn read_anim_refs(
    read: &mut CountingReader<impl Read>,
    count: u8,
) -> Result<Vec<AnimRef>> {
    let mut seen = HashSet::with_capacity(usize::from(count));
    // the first entry... is not zero! as this is not a node list.
    // there's one anim ref per CALL_ANIMATION, and there may be duplicates to
    // the same anim since multiple calls might need to be ordered
    (0..count)
        .map(|index| {
            trace!("Reading anim def ref {}", index);
            let anim_ref_c: AnimRefC = read.read_struct()?;

            let ref_ty = assert_that!(
                "anim def anim ref field 64",
                enum anim_ref_c.ref_ty,
                read.prev + 64
            )?;

            assert_that!(
                "anim def anim ref pointer",
                anim_ref_c.ptr == Ptr::NULL,
                read.prev + 68
            )?;

            match ref_ty {
                AnimRefType::CallAnimation => {
                    // these are full of garbage
                    let (name, name_pad) = assert_utf8("anim ref name", read.prev + 0, || {
                        anim_ref_c.name.to_str_garbage()
                    })?;
                    // this is unfortunately expected
                    if !seen.insert(name.clone()) {
                        log::trace!("anim def duplicate anim ref `{}`", name);
                    }
                    Ok(AnimRef::CallAnimation(AnimRefCallAnimation {
                        name,
                        name_pad,
                    }))
                }
                AnimRefType::CallObjectConnector => {
                    // these are full of garbage
                    let (name, local_name) = anim_ref_c.name.split();
                    let (name, name_pad) =
                        assert_utf8("anim ref name", read.prev + 0, || name.to_str_garbage())?;
                    let (local_name, local_name_pad) =
                        assert_utf8("anim ref local name", read.prev + 32, || {
                            local_name.to_str_garbage()
                        })?;
                    // TODO: not sure if this should use a compound key?
                    if !seen.insert(name.clone()) {
                        log::error!("anim def duplicate anim ref `{}`.`{}`", name, local_name);
                    }
                    Ok(AnimRef::CallObjectConnector(AnimRefCallObjectConnector {
                        name,
                        name_pad,
                        local_name,
                        local_name_pad,
                    }))
                }
            }
        })
        .collect()
}
