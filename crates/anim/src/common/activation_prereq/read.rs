use super::{ActivPrereqAnimC, ActivPrereqHeaderC, ActivPrereqObjC, ActivPrereqType};
use log::trace;
use mech3ax_api_types::anim::{
    ActivationPrerequisite, PrerequisiteAnimation, PrerequisiteObject, PrerequisiteParent,
};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{Result, assert_that};
use mech3ax_types::Ptr;
use std::io::Read;

pub(crate) fn read_activ_prereqs(
    read: &mut CountingReader<impl Read>,
    count: u8,
    is_pm: bool,
) -> Result<Vec<ActivationPrerequisite>> {
    (0..count)
        .map(|index| {
            trace!("Reading activation prerequisite {}", index);
            read_activ_prereq(read, is_pm)
        })
        .collect()
}

fn read_activ_prereq(
    read: &mut CountingReader<impl Read>,
    is_pm: bool,
) -> Result<ActivationPrerequisite> {
    let header: ActivPrereqHeaderC = read.read_struct()?;

    let required = !assert_that!("activ prereq optional", bool header.opt, read.prev - 4)?;
    let prereq_type = assert_that!("activ prereq type", enum header.ty, read.prev)?;

    match prereq_type {
        ActivPrereqType::Animation => {
            let prereq = read_activ_prereq_anim(read, required)?;
            Ok(ActivationPrerequisite::Animation(prereq))
        }
        ActivPrereqType::Object => {
            let prereq = read_activ_prereq_object(read, required, is_pm)?;
            Ok(ActivationPrerequisite::Object(prereq))
        }
        ActivPrereqType::Parent => {
            let prereq = read_activ_prereq_parent(read, required, is_pm)?;
            Ok(ActivationPrerequisite::Parent(prereq))
        }
    }
}

fn read_activ_prereq_anim(
    read: &mut CountingReader<impl Read>,
    required: bool,
) -> Result<PrerequisiteAnimation> {
    let prereq: ActivPrereqAnimC = read.read_struct()?;

    let name = assert_utf8("activ prereq anim name", read.prev + 0, || {
        prereq.name.to_str_padded()
    })?;
    assert_that!(
        "activ prereq anim field 32",
        prereq.zero32 == 0,
        read.prev + 32
    )?;
    assert_that!(
        "activ prereq anim field 36",
        prereq.zero36 == 0,
        read.prev + 36
    )?;

    Ok(PrerequisiteAnimation { name, required })
}

fn read_activ_prereq_object(
    read: &mut CountingReader<impl Read>,
    required: bool,
    is_pm: bool,
) -> Result<PrerequisiteObject> {
    let prereq: ActivPrereqObjC = read.read_struct()?;

    if is_pm {
        assert_that!("activ prereq obj active", prereq.active in [0, 3], read.prev + 0)?;
        assert_that!(
            "activ prereq obj pointer",
            prereq.ptr == Ptr::INVALID,
            read.prev + 36
        )?;
    } else {
        assert_that!("activ prereq obj active", prereq.active in [0, 1], read.prev + 0)?;
        assert_that!(
            "activ prereq obj pointer",
            prereq.ptr != Ptr::NULL,
            read.prev + 36
        )?;
    }
    let active = prereq.active != 0;

    let name = assert_utf8("activ prereq obj name", read.prev + 4, || {
        prereq.name.to_str_padded()
    })?;

    Ok(PrerequisiteObject {
        name,
        required,
        active,
        ptr: prereq.ptr.0,
    })
}

fn read_activ_prereq_parent(
    read: &mut CountingReader<impl Read>,
    required: bool,
    is_pm: bool,
) -> Result<PrerequisiteParent> {
    let prereq: ActivPrereqObjC = read.read_struct()?;

    if is_pm {
        assert_that!("activ prereq parent active", prereq.active in [0, 2], read.prev + 0)?;
        assert_that!(
            "activ prereq parent pointer",
            prereq.ptr == Ptr::INVALID,
            read.prev + 36
        )?;
    } else {
        assert_that!(
            "activ prereq parent active",
            prereq.active == 0,
            read.prev + 0
        )?;
        assert_that!(
            "activ prereq parent pointer",
            prereq.ptr != Ptr::NULL,
            read.prev + 36
        )?;
    }
    let active = prereq.active != 0;

    let name = assert_utf8("activ prereq parent name", read.prev + 4, || {
        prereq.name.to_str_padded()
    })?;

    Ok(PrerequisiteParent {
        name,
        required,
        active,
        ptr: prereq.ptr.0,
    })
}
