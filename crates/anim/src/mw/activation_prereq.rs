use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::{ActivationPrereq, PrereqAnimation, PrereqObject, PrereqParent};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{impl_as_bytes, primitive_enum, Ascii, Bool32, Maybe};
use std::io::{Read, Write};

primitive_enum! {
    enum ActivPrereqType: u32 {
        Animation = 1,
        Object = 2,
        Parent = 3,
    }
}

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ActivPrereqAnimC {
    name: Ascii<32>, // 00
    zero32: u32,     // 32
    zero36: u32,     // 36
}
impl_as_bytes!(ActivPrereqAnimC, 40);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ActivPrereqObjC {
    active: Bool32,  // 00
    name: Ascii<32>, // 32
    pointer: u32,    // 36
}
impl_as_bytes!(ActivPrereqObjC, 40);

fn read_activ_prereq_anim(read: &mut CountingReader<impl Read>) -> Result<ActivationPrereq> {
    let prereq: ActivPrereqAnimC = read.read_struct()?;
    let name = assert_utf8("anim def activ prereq a name", read.prev + 0, || {
        prereq.name.to_str_padded()
    })?;
    assert_that!(
        "anim def activ prereq a field 32",
        prereq.zero32 == 0,
        read.prev + 32
    )?;
    assert_that!(
        "anim def activ prereq a field 36",
        prereq.zero36 == 0,
        read.prev + 36
    )?;
    Ok(ActivationPrereq::Animation(PrereqAnimation { name }))
}

fn read_activ_prereq_parent(
    read: &mut CountingReader<impl Read>,
    required: bool,
) -> Result<ActivationPrereq> {
    let prereq: ActivPrereqObjC = read.read_struct()?;
    assert_that!(
        "anim def activ prereq p active",
        prereq.active == Bool32::FALSE,
        read.prev + 0
    )?;
    let name = assert_utf8("anim def activ prereq p name", read.prev + 4, || {
        prereq.name.to_str_padded()
    })?;
    assert_that!(
        "anim def activ prereq p pointer",
        prereq.pointer != 0,
        read.prev + 36
    )?;
    Ok(ActivationPrereq::Parent(PrereqParent {
        name,
        required,
        active: false,
        pointer: prereq.pointer,
    }))
}

fn read_activ_prereq_object(
    read: &mut CountingReader<impl Read>,
    required: bool,
) -> Result<ActivationPrereq> {
    let prereq: ActivPrereqObjC = read.read_struct()?;
    let active = assert_that!("anim def activ prereq o active", bool prereq.active, read.prev + 0)?;
    let name = assert_utf8("anim def activ prereq o name", read.prev + 4, || {
        prereq.name.to_str_padded()
    })?;
    assert_that!(
        "anim def activ prereq o pointer",
        prereq.pointer != 0,
        read.prev + 36
    )?;
    Ok(ActivationPrereq::Object(PrereqObject {
        name,
        required,
        active,
        pointer: prereq.pointer,
    }))
}

fn read_activ_prereq(read: &mut CountingReader<impl Read>) -> Result<ActivationPrereq> {
    let optional = Bool32::new(read.read_u32()?);
    let required = !assert_that!("anim def activ prereq optional", bool optional, read.prev)?;
    let prereq_type_raw = Maybe::new(read.read_u32()?);
    let prereq_type = assert_that!("activ prereq type", enum prereq_type_raw, read.prev)?;
    match prereq_type {
        ActivPrereqType::Animation => {
            assert_that!(
                "anim def activ prereq required",
                required == true,
                read.prev - 4
            )?;
            read_activ_prereq_anim(read)
        }
        ActivPrereqType::Parent => read_activ_prereq_parent(read, required),
        ActivPrereqType::Object => read_activ_prereq_object(read, required),
    }
}

pub fn read_activ_prereqs(
    read: &mut CountingReader<impl Read>,
    count: u8,
) -> Result<Vec<ActivationPrereq>> {
    (0..count).map(|_| read_activ_prereq(read)).collect()
}

fn write_activ_prereq_anim(write: &mut CountingWriter<impl Write>, name: &str) -> Result<()> {
    let fill = Ascii::from_str_padded(name);
    // always required (not optional)
    write.write_u32(false.into())?;
    write.write_u32(ActivPrereqType::Animation as u32)?;
    write.write_struct(&ActivPrereqAnimC {
        name: fill,
        zero32: 0,
        zero36: 0,
    })?;
    Ok(())
}

fn write_activ_prereq_object(
    write: &mut CountingWriter<impl Write>,
    object: &PrereqObject,
    prereq_type: ActivPrereqType,
) -> Result<()> {
    let name = Ascii::from_str_padded(&object.name);
    write.write_u32((!object.required).into())?;
    write.write_u32(prereq_type as u32)?;
    write.write_struct(&ActivPrereqObjC {
        active: object.active.into(),
        name,
        pointer: object.pointer,
    })?;
    Ok(())
}

fn write_activ_prereq_parent(
    write: &mut CountingWriter<impl Write>,
    parent: &PrereqParent,
    prereq_type: ActivPrereqType,
) -> Result<()> {
    let name = Ascii::from_str_padded(&parent.name);
    write.write_u32((!parent.required).into())?;
    write.write_u32(prereq_type as u32)?;
    write.write_struct(&ActivPrereqObjC {
        active: parent.active.into(),
        name,
        pointer: parent.pointer,
    })?;
    Ok(())
}

pub fn write_activ_prereqs(
    write: &mut CountingWriter<impl Write>,
    activ_prereqs: &[ActivationPrereq],
) -> Result<()> {
    for activ_prereq in activ_prereqs {
        match activ_prereq {
            ActivationPrereq::Animation(PrereqAnimation { name }) => {
                write_activ_prereq_anim(write, name)?
            }
            ActivationPrereq::Object(object) => {
                write_activ_prereq_object(write, object, ActivPrereqType::Object)?
            }
            ActivationPrereq::Parent(parent) => {
                write_activ_prereq_parent(write, parent, ActivPrereqType::Parent)?
            }
        }
    }
    Ok(())
}
