use super::types::*;
use crate::assert::{assert_all_zero, assert_utf8};
use crate::io_ext::{CountingReader, WriteHelper};
use crate::size::ReprSize;
use crate::string::{
    bytes_to_c, str_from_c_node_name, str_from_c_padded, str_from_c_partition, str_to_c_node_name,
    str_to_c_padded, str_to_c_partition,
};
use crate::{assert_that, static_assert_size, Result};
use log::trace;
use std::io::{Read, Write};

#[repr(C)]
struct ObjectC {
    name: [u8; 32], // 00
    zero32: u32,    // 32
    unk: [u8; 60],  // 36
}
static_assert_size!(ObjectC, 96);

pub fn read_objects<R: Read>(read: &mut CountingReader<R>, count: u8) -> Result<Vec<NamePad>> {
    trace!("Reading anim def object 0 at {}", read.offset);
    // the first entry is always zero
    let object: ObjectC = read.read_struct()?;
    assert_all_zero("anim def object zero name", read.prev + 0, &object.name)?;
    assert_that!(
        "anim def object zero field 32",
        object.zero32 == 0,
        read.prev + 32
    )?;
    assert_all_zero("anim def object zero unk", read.prev + 0, &object.unk)?;
    (1..count)
        .map(|i| {
            trace!("Reading anim def object {} at {}", i, read.offset);
            let object: ObjectC = read.read_struct()?;
            let name = assert_utf8("anim def object name", read.prev + 0, || {
                str_from_c_node_name(&object.name)
            })?;
            assert_that!(
                "anim def object field 32",
                object.zero32 == 0,
                read.prev + 32
            )?;
            // TODO: this is cheating, but i have no idea how to interpret this data.
            // sometimes it's sensible, e.g. floats. other times, it seems like random
            // garbage.
            let pad = object.unk.to_vec();
            Ok(NamePad { name, pad })
        })
        .collect()
}

pub fn write_objects<W: Write>(write: &mut W, objects: &[NamePad]) -> Result<()> {
    // the first entry is always zero
    write.write_zeros(ObjectC::SIZE)?;
    for object in objects {
        let mut name = [0; 32];
        str_to_c_node_name(&object.name, &mut name);
        let mut unk = [0; 60];
        bytes_to_c(&object.pad, &mut unk);
        write.write_struct(&ObjectC {
            name,
            zero32: 0,
            unk,
        })?;
    }
    Ok(())
}

#[repr(C)]
struct NodeInfoC {
    name: [u8; 32], // 00
    zero32: u32,    // 32
    pointer: u32,   // 36
}
static_assert_size!(NodeInfoC, 40);

pub fn read_nodes<R: Read>(read: &mut CountingReader<R>, count: u8) -> Result<Vec<NamePtr>> {
    trace!("Reading anim def node 0 at {}", read.offset);
    // the first entry is always zero
    let node_info: NodeInfoC = read.read_struct()?;
    assert_all_zero("anim def node zero name", read.prev + 0, &node_info.name)?;
    assert_that!(
        "anim def node zero field 32",
        node_info.zero32 == 0,
        read.prev + 32
    )?;
    assert_that!(
        "anim def node zero pointer",
        node_info.pointer == 0,
        read.prev + 36
    )?;

    (1..count)
        .map(|i| {
            trace!("Reading anim def node {} at {}", i, read.offset);
            let node_info: NodeInfoC = read.read_struct()?;
            let name = assert_utf8("anim def node name", read.prev + 0, || {
                str_from_c_node_name(&node_info.name)
            })?;
            assert_that!(
                "anim def node field 32",
                node_info.zero32 == 0,
                read.prev + 32
            )?;
            assert_that!(
                "anim def node pointer",
                node_info.pointer != 0,
                read.prev + 36
            )?;
            Ok(NamePtr {
                name,
                pointer: node_info.pointer,
            })
        })
        .collect()
}

pub fn write_nodes<W: Write>(write: &mut W, nodes: &[NamePtr]) -> Result<()> {
    // the first entry is always zero
    write.write_zeros(NodeInfoC::SIZE)?;
    for node_info in nodes {
        let mut name = [0; 32];
        str_to_c_node_name(&node_info.name, &mut name);
        write.write_struct(&NodeInfoC {
            name,
            zero32: 0,
            pointer: node_info.pointer,
        })?;
    }
    Ok(())
}

#[repr(C)]
struct ReaderLookupC {
    name: [u8; 32], // 00
    flags: u32,     // 32
    pointer: u32,   // 36
    zero40: u32,    // 40
}
static_assert_size!(ReaderLookupC, 44);

pub fn read_lights<R: Read>(read: &mut CountingReader<R>, count: u8) -> Result<Vec<NamePtr>> {
    trace!("Reading anim def light 0 at {}", read.offset);
    // the first entry is always zero
    let light: ReaderLookupC = read.read_struct()?;
    assert_all_zero("anim def light zero name", read.prev + 0, &light.name)?;
    assert_that!(
        "anim def node light field 32",
        light.flags == 0,
        read.prev + 32
    )?;
    assert_that!(
        "anim def node light pointer",
        light.pointer == 0,
        read.prev + 36
    )?;
    assert_that!(
        "anim def node light field 40",
        light.zero40 == 0,
        read.prev + 40
    )?;

    (1..count)
        .map(|i| {
            trace!("Reading anim def light {} at {}", i, read.offset);
            let light: ReaderLookupC = read.read_struct()?;
            let name = assert_utf8("anim def light name", read.prev + 0, || {
                str_from_c_node_name(&light.name)
            })?;
            assert_that!("anim def light field 32", light.flags == 0, read.prev + 32)?;
            assert_that!("anim def light pointer", light.pointer != 0, read.prev + 36)?;
            // if this were non-zero, it would cause the light to be removed instead of added (???)
            assert_that!("anim def light field 40", light.zero40 == 0, read.prev + 40)?;
            Ok(NamePtr {
                name,
                pointer: light.pointer,
            })
        })
        .collect()
}

pub fn write_lights<W: Write>(write: &mut W, lights: &[NamePtr]) -> Result<()> {
    // the first entry is always zero
    write.write_zeros(ReaderLookupC::SIZE)?;
    for light in lights {
        let mut name = [0; 32];
        str_to_c_node_name(&light.name, &mut name);
        write.write_struct(&ReaderLookupC {
            name,
            flags: 0,
            pointer: light.pointer,
            zero40: 0,
        })?;
    }
    Ok(())
}

pub fn read_puffers<R: Read>(read: &mut CountingReader<R>, count: u8) -> Result<Vec<NamePtrFlags>> {
    trace!("Reading anim def puffer 0 at {}", read.offset);
    // the first entry is always zero
    let puffer: ReaderLookupC = read.read_struct()?;
    assert_all_zero("anim def puffer zero name", read.prev + 0, &puffer.name)?;
    assert_that!(
        "anim def node puffer flags",
        puffer.flags == 0,
        read.prev + 32
    )?;
    assert_that!(
        "anim def node puffer pointer",
        puffer.pointer == 0,
        read.prev + 36
    )?;
    assert_that!(
        "anim def node puffer field 40",
        puffer.zero40 == 0,
        read.prev + 40
    )?;

    (1..count)
        .map(|i| {
            trace!("Reading anim def puffer {} at {}", i, read.offset);
            let puffer: ReaderLookupC = read.read_struct()?;
            let name = assert_utf8("anim def puffer name", read.prev + 0, || {
                str_from_c_padded(&puffer.name)
            })?;
            // TODO: what does this flag mean?
            // this is something the code does, but i'm not sure why
            // some of these values make decent floating point numbers
            let flags = puffer.flags & 0x00FFFFFF;
            assert_that!("anim def node puffer flags", flags == 0, read.prev + 32)?;
            let flags = puffer.flags >> 24;
            assert_that!(
                "anim def puffer pointer",
                puffer.pointer != 0,
                read.prev + 36
            )?;
            assert_that!(
                "anim def puffer field 40",
                puffer.zero40 == 0,
                read.prev + 40
            )?;
            Ok(NamePtrFlags {
                name,
                pointer: puffer.pointer,
                flags,
            })
        })
        .collect()
}

pub fn write_puffers<W: Write>(write: &mut W, puffers: &[NamePtrFlags]) -> Result<()> {
    // the first entry is always zero
    write.write_zeros(ReaderLookupC::SIZE)?;
    for puffer in puffers {
        let mut name = [0; 32];
        str_to_c_padded(&puffer.name, &mut name);
        let flags = puffer.flags << 24;
        write.write_struct(&ReaderLookupC {
            name,
            flags,
            pointer: puffer.pointer,
            zero40: 0,
        })?;
    }
    Ok(())
}

pub fn read_dynamic_sounds<R: Read>(
    read: &mut CountingReader<R>,
    count: u8,
) -> Result<Vec<NamePtr>> {
    trace!("Reading anim def dynamic sound 0 at {}", read.offset);
    // the first entry is always zero
    let dynamic_sound: ReaderLookupC = read.read_struct()?;
    assert_all_zero(
        "anim def dynamic sound zero name",
        read.prev + 0,
        &dynamic_sound.name,
    )?;
    assert_that!(
        "anim def node dynamic sound field 32",
        dynamic_sound.flags == 0,
        read.prev + 32
    )?;
    assert_that!(
        "anim def node dynamic sound pointer",
        dynamic_sound.pointer == 0,
        read.prev + 36
    )?;
    assert_that!(
        "anim def node dynamic sound field 40",
        dynamic_sound.zero40 == 0,
        read.prev + 40
    )?;

    (1..count)
        .map(|i| {
            trace!("Reading anim def dynamic sound {} at {}", i, read.offset);
            let dynamic_sound: ReaderLookupC = read.read_struct()?;
            let name = assert_utf8("anim def dynamic sound name", read.prev + 0, || {
                str_from_c_node_name(&dynamic_sound.name)
            })?;
            assert_that!(
                "anim def dynamic sound field 32",
                dynamic_sound.flags == 0,
                read.prev + 32
            )?;
            assert_that!(
                "anim def dynamic sound pointer",
                dynamic_sound.pointer != 0,
                read.prev + 36
            )?;
            assert_that!(
                "anim def dynamic sound field 40",
                dynamic_sound.flags == 0,
                read.prev + 40
            )?;
            Ok(NamePtr {
                name,
                pointer: dynamic_sound.pointer,
            })
        })
        .collect()
}

pub fn write_dynamic_sounds<W: Write>(write: &mut W, dynamic_sounds: &[NamePtr]) -> Result<()> {
    // the first entry is always zero
    write.write_zeros(ReaderLookupC::SIZE)?;
    for dynamic_sound in dynamic_sounds {
        let mut name = [0; 32];
        str_to_c_node_name(&dynamic_sound.name, &mut name);
        write.write_struct(&ReaderLookupC {
            name,
            flags: 0,
            pointer: dynamic_sound.pointer,
            zero40: 0,
        })?;
    }
    Ok(())
}

#[repr(C)]
struct StaticSoundC {
    name: [u8; 32], // 00
    zero32: u32,    // 32
}
static_assert_size!(StaticSoundC, 36);

pub fn read_static_sounds<R: Read>(
    read: &mut CountingReader<R>,
    count: u8,
) -> Result<Vec<NamePad>> {
    trace!("Reading anim def static sound 0 at {}", read.offset);
    // the first entry is always zero
    let static_sound: StaticSoundC = read.read_struct()?;
    assert_all_zero(
        "anim def static sound zero name",
        read.prev + 0,
        &static_sound.name,
    )?;
    assert_that!(
        "anim def static sound zero field 32",
        static_sound.zero32 == 0,
        read.prev + 32
    )?;
    (1..count)
        .map(|i| {
            trace!("Reading anim def static sound {} at {}", i, read.offset);
            let static_sound: StaticSoundC = read.read_struct()?;
            let (name, pad) = assert_utf8("anim def static sound name", read.prev + 0, || {
                str_from_c_partition(&static_sound.name)
            })?;
            assert_that!(
                "anim def static sound field 32",
                static_sound.zero32 == 0,
                read.prev + 32
            )?;
            Ok(NamePad { name, pad })
        })
        .collect()
}

pub fn write_static_sounds<W: Write>(write: &mut W, static_sounds: &[NamePad]) -> Result<()> {
    // the first entry is always zero
    write.write_zeros(StaticSoundC::SIZE)?;
    for static_sound in static_sounds {
        let mut name = [0; 32];
        str_to_c_partition(&static_sound.name, &static_sound.pad, &mut name);
        write.write_struct(&StaticSoundC { name, zero32: 0 })?;
    }
    Ok(())
}

#[repr(C)]
struct AnimRefC {
    name: [u8; 64], // 00
    zero64: u32,    // 64
    zero68: u32,    // 68
}
static_assert_size!(AnimRefC, 72);

pub fn read_anim_refs<R: Read>(read: &mut CountingReader<R>, count: u8) -> Result<Vec<NamePad>> {
    // the first entry... is not zero! as this is not a node list
    // there's one anim ref per CALL_ANIMATION, and there may be duplicates to
    // the same anim since multiple calls might need to be ordered
    (0..count)
        .map(|i| {
            trace!("Reading anim def anim ref {} at {}", i, read.offset);
            let anim_ref: AnimRefC = read.read_struct()?;
            // a bunch of these values are properly zero-terminated at 32 and beyond,
            // but not all... i suspect a lack of memset
            let (name, pad) = assert_utf8("anim def anim ref name", read.prev + 0, || {
                str_from_c_partition(&anim_ref.name)
            })?;
            assert_that!(
                "anim def anim ref zero field 64",
                anim_ref.zero64 == 0,
                read.prev + 64
            )?;
            assert_that!(
                "anim def anim ref zero field 68",
                anim_ref.zero68 == 0,
                read.prev + 68
            )?;
            Ok(NamePad { name, pad })
        })
        .collect()
}

pub fn write_anim_refs<W: Write>(write: &mut W, anim_refs: &[NamePad]) -> Result<()> {
    // the first entry... is not zero! as this is not a node list
    for anim_ref in anim_refs {
        let mut name = [0; 64];
        str_to_c_partition(&anim_ref.name, &anim_ref.pad, &mut name);
        write.write_struct(&AnimRefC {
            name,
            zero64: 0,
            zero68: 0,
        })?;
    }
    Ok(())
}
