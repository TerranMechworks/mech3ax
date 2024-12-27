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
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::Result;
use mech3ax_types::{Ascii, Bytes, EnumerateEx as _, Hex, Ptr};
use std::io::Write;

pub(crate) fn bin_to_affine(slice: &[u8]) -> AffineMatrix {
    let src = Bytes::<48>::from_slice(slice);
    let mut v = AffineMatrix::default();
    let bytes: &mut [u8; 48] = bytemuck::must_cast_mut(&mut v);
    bytes.copy_from_slice(&src);
    v
}

pub(crate) fn write_objects(
    write: &mut CountingWriter<impl Write>,
    objects: &[ObjectRef],
) -> Result<()> {
    trace!("Writing anim def object 0");
    // the first entry is always zero
    let object_c = ObjectRefC::default();
    write.write_struct(&object_c)?;

    for (index, object) in objects.iter().enumerate_one() {
        trace!("Writing anim def object {}", index);

        let name = if object.name == ABORT_TEST_STR {
            debug!("anim def object name `abort_test` fixup");
            ABORT_TEST_RAW
        } else {
            Ascii::from_str_node_name(&object.name)
        };
        let affine = bin_to_affine(&object.affine);

        let object_c = ObjectRefC {
            name,
            zero32: 0,
            ptr: Ptr(object.ptr.unwrap_or(0)),
            flags: Hex(object.flags),
            flags_merged: Hex(object.flags_merged.unwrap_or(0)),
            affine,
        };
        write.write_struct(&object_c)?;
    }
    Ok(())
}

pub(crate) fn write_nodes(write: &mut CountingWriter<impl Write>, nodes: &[NodeRef]) -> Result<()> {
    trace!("Writing anim def node 0");
    // the first entry is always zero
    let node_c = NodeRefC::default();
    write.write_struct(&node_c)?;

    for (index, node) in nodes.iter().enumerate_one() {
        trace!("Writing anim def node {}", index);

        let name = if node.name == ABORT_TEST_STR {
            debug!("anim def node name `abort_test` fixup");
            ABORT_TEST_RAW
        } else {
            Ascii::from_str_node_name(&node.name)
        };
        let node_c = NodeRefC {
            name,
            zero32: 0,
            ptr: node.ptr.into(),
        };
        write.write_struct(&node_c)?;
    }
    Ok(())
}

pub(crate) fn write_lights(
    write: &mut CountingWriter<impl Write>,
    lights: &[LightRef],
) -> Result<()> {
    trace!("Writing anim def light 0");
    // the first entry is always zero
    let light_c = LightRefC::default();
    write.write_struct(&light_c)?;

    for (index, light) in lights.iter().enumerate_one() {
        trace!("Writing anim def light {}", index);

        let name = Ascii::from_str_node_name(&light.name);
        let light_c = LightRefC {
            name,
            flags: 0,
            ptr: light.ptr.into(),
            in_world: 0,
        };
        write.write_struct(&light_c)?;
    }
    Ok(())
}

pub(crate) fn write_puffers(
    write: &mut CountingWriter<impl Write>,
    puffers: &[PufferRef],
) -> Result<()> {
    trace!("Writing anim def puffer 0");
    // the first entry is always zero
    let puffer_c = PufferRefC::default();
    write.write_struct(&puffer_c)?;

    for (index, puffer) in puffers.iter().enumerate_one() {
        trace!("Writing anim def puffer {}", index);

        let name = Ascii::from_str_padded(&puffer.name);
        let flags = (puffer.flags as u32) << 24;

        let puffer_c = PufferRefC {
            name,
            flags,
            ptr: puffer.ptr.into(),
            in_world: 0,
        };
        write.write_struct(&puffer_c)?;
    }
    Ok(())
}

pub(crate) fn write_dynamic_sounds(
    write: &mut CountingWriter<impl Write>,
    dynamic_sounds: &[DynamicSoundRef],
) -> Result<()> {
    trace!("Writing anim def dyn sound 0");
    // the first entry is always zero
    let sound_c = DynamicSoundRefC::default();
    write.write_struct(&sound_c)?;

    for (index, dynamic_sound) in dynamic_sounds.iter().enumerate_one() {
        trace!("Writing anim def dyn sound {}", index);

        let name = Ascii::from_str_node_name(&dynamic_sound.name);
        let sound_c = DynamicSoundRefC {
            name,
            flags: 0,
            ptr: dynamic_sound.ptr.into(),
            in_world: 0,
        };
        write.write_struct(&sound_c)?;
    }
    Ok(())
}

pub(crate) fn write_static_sounds(
    write: &mut CountingWriter<impl Write>,
    static_sounds: &[StaticSoundRef],
) -> Result<()> {
    trace!("Writing anim def stc sound 0");
    // the first entry is always zero
    let sound_c = StaticSoundRefC::default();
    write.write_struct(&sound_c)?;

    for (index, static_sound) in static_sounds.iter().enumerate_one() {
        trace!("Writing anim def stc sound {}", index);

        let name = Ascii::from_str_garbage(&static_sound.name, &static_sound.pad);
        let sound_c = StaticSoundRefC { name, zero32: 0 };
        write.write_struct(&sound_c)?;
    }
    Ok(())
}

pub(crate) fn write_effects(
    write: &mut CountingWriter<impl Write>,
    effects: &[EffectRef],
) -> Result<()> {
    trace!("Writing anim def effect 0");
    let effect_c = EffectRefC::default();
    write.write_struct(&effect_c)?;

    for (index, effect) in effects.iter().enumerate_one() {
        trace!("Writing anim def effect {}", index);

        let name = Ascii::from_str_garbage(&effect.name, &effect.pad);
        let effect_c = EffectRefC {
            name,
            index: effect.index,
        };
        write.write_struct(&effect_c)?;
    }
    Ok(())
}

pub(crate) fn write_anim_refs(
    write: &mut CountingWriter<impl Write>,
    anim_refs: &[AnimRef],
) -> Result<()> {
    // the first entry... is not zero! as this is not a node list
    for (index, anim_ref) in anim_refs.iter().enumerate() {
        trace!("Writing anim def ref {}", index);

        let anim_ref_c = match anim_ref {
            AnimRef::CallAnimation(AnimRefCallAnimation { name, name_pad }) => {
                let name = Ascii::from_str_garbage(name, name_pad);
                AnimRefC {
                    name,
                    ref_ty: AnimRefType::CallAnimation.maybe(),
                    ptr: Ptr::NULL,
                }
            }
            AnimRef::CallObjectConnector(AnimRefCallObjectConnector {
                name,
                name_pad,
                local_name,
                local_name_pad,
            }) => {
                let name = Ascii::from_str_garbage(name, name_pad);
                let local_name = Ascii::from_str_garbage(local_name, local_name_pad);
                let name = Ascii::combine(&name, &local_name);
                AnimRefC {
                    name,
                    ref_ty: AnimRefType::CallObjectConnector.maybe(),
                    ptr: Ptr::NULL,
                }
            }
        };
        write.write_struct(&anim_ref_c)?;
    }
    Ok(())
}
