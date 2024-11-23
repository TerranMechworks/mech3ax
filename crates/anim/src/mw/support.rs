use bytemuck::{AnyBitPattern, NoUninit};
use log::trace;
use mech3ax_api_types::anim::{NamePad, NamePtr, NamePtrFlags};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{impl_as_bytes, AsBytes as _, Ascii, Bytes};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ObjectC {
    name: Ascii<32>, // 00
    zero32: u32,     // 32
    unk: Bytes<60>,  // 36
}
impl_as_bytes!(ObjectC, 96);

pub fn read_objects(read: &mut CountingReader<impl Read>, count: u8) -> Result<Vec<NamePad>> {
    trace!("Reading anim def object 0 at {}", read.offset);
    // the first entry is always zero
    let object: ObjectC = read.read_struct()?;
    assert_that!("anim def object zero name", zero object.name, read.prev + 0)?;
    assert_that!(
        "anim def object zero field 32",
        object.zero32 == 0,
        read.prev + 32
    )?;
    assert_that!("anim def object zero unk", zero object.unk, read.prev + 0)?;
    (1..count)
        .map(|i| {
            trace!("Reading anim def object {} at {}", i, read.offset);
            let object: ObjectC = read.read_struct()?;
            let name = assert_utf8("anim def object name", read.prev + 0, || {
                object.name.to_str_node_name()
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

pub fn write_objects(write: &mut CountingWriter<impl Write>, objects: &[NamePad]) -> Result<()> {
    // the first entry is always zero
    write.write_zeros(ObjectC::SIZE)?;
    for object in objects {
        let name = Ascii::from_str_node_name(&object.name);
        let unk = Bytes::from_slice(&object.pad);
        write.write_struct(&ObjectC {
            name,
            zero32: 0,
            unk,
        })?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct NodeInfoC {
    name: Ascii<32>, // 00
    zero32: u32,     // 32
    pointer: u32,    // 36
}
impl_as_bytes!(NodeInfoC, 40);

pub fn read_nodes(read: &mut CountingReader<impl Read>, count: u8) -> Result<Vec<NamePtr>> {
    trace!("Reading anim def node 0 at {}", read.offset);
    // the first entry is always zero
    let node_info: NodeInfoC = read.read_struct()?;
    assert_that!("anim def node zero name", zero node_info.name, read.prev + 0)?;
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
                node_info.name.to_str_node_name()
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

pub fn write_nodes(write: &mut CountingWriter<impl Write>, nodes: &[NamePtr]) -> Result<()> {
    // the first entry is always zero
    write.write_zeros(NodeInfoC::SIZE)?;
    for node_info in nodes {
        let name = Ascii::from_str_node_name(&node_info.name);
        write.write_struct(&NodeInfoC {
            name,
            zero32: 0,
            pointer: node_info.pointer,
        })?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ReaderLookupC {
    name: Ascii<32>, // 00
    flags: u32,      // 32
    pointer: u32,    // 36
    in_world: u32,   // 40
}
impl_as_bytes!(ReaderLookupC, 44);

pub fn read_lights(read: &mut CountingReader<impl Read>, count: u8) -> Result<Vec<NamePtr>> {
    trace!("Reading anim def light 0 at {}", read.offset);
    // the first entry is always zero
    let light: ReaderLookupC = read.read_struct()?;
    assert_that!("anim def light zero name", zero light.name, read.prev + 0)?;
    assert_that!(
        "anim def node light flags",
        light.flags == 0,
        read.prev + 32
    )?;
    assert_that!(
        "anim def node light pointer",
        light.pointer == 0,
        read.prev + 36
    )?;
    assert_that!(
        "anim def node light in world",
        light.in_world == 0,
        read.prev + 40
    )?;

    (1..count)
        .map(|i| {
            trace!("Reading anim def light {} at {}", i, read.offset);
            let light: ReaderLookupC = read.read_struct()?;
            let name = assert_utf8("anim def light name", read.prev + 0, || {
                light.name.to_str_node_name()
            })?;
            assert_that!("anim def light flags", light.flags == 0, read.prev + 32)?;
            assert_that!("anim def light pointer", light.pointer != 0, read.prev + 36)?;
            // if this were non-zero, it would cause the light to be removed instead of added (???)
            assert_that!(
                "anim def light in world",
                light.in_world == 0,
                read.prev + 40
            )?;
            Ok(NamePtr {
                name,
                pointer: light.pointer,
            })
        })
        .collect()
}

pub fn write_lights(write: &mut CountingWriter<impl Write>, lights: &[NamePtr]) -> Result<()> {
    // the first entry is always zero
    write.write_zeros(ReaderLookupC::SIZE)?;
    for light in lights {
        let name = Ascii::from_str_node_name(&light.name);
        write.write_struct(&ReaderLookupC {
            name,
            flags: 0,
            pointer: light.pointer,
            in_world: 0,
        })?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct PufferRefC {
    name: Ascii<32>, // 00
    flags: u32,      // 32
    pointer: u32,    // 36
    zero40: u32,     // 40
}
impl_as_bytes!(PufferRefC, 44);

pub fn read_puffers(read: &mut CountingReader<impl Read>, count: u8) -> Result<Vec<NamePtrFlags>> {
    trace!("Reading anim def puffer 0 at {}", read.offset);
    // the first entry is always zero
    let mut puffer = [0; PufferRefC::SIZE as usize];
    read.read_exact(&mut puffer)?;
    assert_that!("anim def puffer zero", zero puffer, read.prev + 0)?;

    (1..count)
        .map(|i| {
            trace!("Reading anim def puffer {} at {}", i, read.offset);
            let puffer: PufferRefC = read.read_struct()?;
            let name = assert_utf8("anim def puffer name", read.prev + 0, || {
                puffer.name.to_str_padded()
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

pub fn write_puffers(
    write: &mut CountingWriter<impl Write>,
    puffers: &[NamePtrFlags],
) -> Result<()> {
    // the first entry is always zero
    write.write_zeros(ReaderLookupC::SIZE)?;
    for puffer in puffers {
        let name = Ascii::from_str_padded(&puffer.name);
        let flags = puffer.flags << 24;
        write.write_struct(&ReaderLookupC {
            name,
            flags,
            pointer: puffer.pointer,
            in_world: 0,
        })?;
    }
    Ok(())
}

pub fn read_dynamic_sounds(
    read: &mut CountingReader<impl Read>,
    count: u8,
) -> Result<Vec<NamePtr>> {
    trace!("Reading anim def dynamic sound 0 at {}", read.offset);
    // the first entry is always zero
    let dynamic_sound: ReaderLookupC = read.read_struct()?;
    assert_that!(
        "anim def dynamic sound zero name",
        zero dynamic_sound.name,
        read.prev + 0
    )?;
    assert_that!(
        "anim def node dynamic sound flags",
        dynamic_sound.flags == 0,
        read.prev + 32
    )?;
    assert_that!(
        "anim def node dynamic sound pointer",
        dynamic_sound.pointer == 0,
        read.prev + 36
    )?;
    assert_that!(
        "anim def node dynamic sound in world",
        dynamic_sound.in_world == 0,
        read.prev + 40
    )?;

    (1..count)
        .map(|i| {
            trace!("Reading anim def dynamic sound {} at {}", i, read.offset);
            let dynamic_sound: ReaderLookupC = read.read_struct()?;
            let name = assert_utf8("anim def dynamic sound name", read.prev + 0, || {
                dynamic_sound.name.to_str_node_name()
            })?;
            assert_that!(
                "anim def dynamic sound flags",
                dynamic_sound.flags == 0,
                read.prev + 32
            )?;
            assert_that!(
                "anim def dynamic sound pointer",
                dynamic_sound.pointer != 0,
                read.prev + 36
            )?;
            assert_that!(
                "anim def dynamic sound in world",
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

pub fn write_dynamic_sounds(
    write: &mut CountingWriter<impl Write>,
    dynamic_sounds: &[NamePtr],
) -> Result<()> {
    // the first entry is always zero
    write.write_zeros(ReaderLookupC::SIZE)?;
    for dynamic_sound in dynamic_sounds {
        let name = Ascii::from_str_node_name(&dynamic_sound.name);
        write.write_struct(&ReaderLookupC {
            name,
            flags: 0,
            pointer: dynamic_sound.pointer,
            in_world: 0,
        })?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct StaticSoundC {
    name: Ascii<32>, // 00
    zero32: u32,     // 32
}
impl_as_bytes!(StaticSoundC, 36);

pub fn read_static_sounds(read: &mut CountingReader<impl Read>, count: u8) -> Result<Vec<NamePad>> {
    trace!("Reading anim def static sound 0 at {}", read.offset);
    // the first entry is always zero
    let static_sound: StaticSoundC = read.read_struct()?;
    assert_that!(
        "anim def static sound zero name",
        zero static_sound.name,
        read.prev + 0
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
                static_sound.name.to_str_garbage()
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

pub fn write_static_sounds(
    write: &mut CountingWriter<impl Write>,
    static_sounds: &[NamePad],
) -> Result<()> {
    // the first entry is always zero
    write.write_zeros(StaticSoundC::SIZE)?;
    for static_sound in static_sounds {
        let name = Ascii::from_str_garbage(&static_sound.name, &static_sound.pad);
        write.write_struct(&StaticSoundC { name, zero32: 0 })?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct AnimRefC {
    name: Ascii<64>, // 00
    zero64: u32,     // 64
    pointer: u32,    // 68
}
impl_as_bytes!(AnimRefC, 72);

pub fn read_anim_refs(read: &mut CountingReader<impl Read>, count: u8) -> Result<Vec<NamePad>> {
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
                anim_ref.name.to_str_garbage()
            })?;
            assert_that!(
                "anim def anim ref field 64",
                anim_ref.zero64 == 0,
                read.prev + 64
            )?;
            assert_that!(
                "anim def anim ref pointer",
                anim_ref.pointer == 0,
                read.prev + 68
            )?;
            Ok(NamePad { name, pad })
        })
        .collect()
}

pub fn write_anim_refs(
    write: &mut CountingWriter<impl Write>,
    anim_refs: &[NamePad],
) -> Result<()> {
    // the first entry... is not zero! as this is not a node list
    for anim_ref in anim_refs {
        let name = Ascii::from_str_garbage(&anim_ref.name, &anim_ref.pad);
        write.write_struct(&AnimRefC {
            name,
            zero64: 0,
            pointer: 0,
        })?;
    }
    Ok(())
}
