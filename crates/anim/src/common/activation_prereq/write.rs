use super::{ActivPrereqAnimC, ActivPrereqHeaderC, ActivPrereqObjC, ActivPrereqType};
use log::trace;
use mech3ax_api_types::anim::{
    ActivationPrerequisite, PrerequisiteAnimation, PrerequisiteObject, PrerequisiteParent,
};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::Result;
use mech3ax_types::{Ascii, Ptr};
use std::io::Write;

pub(crate) fn write_activ_prereqs(
    write: &mut CountingWriter<impl Write>,
    activ_prereqs: &[ActivationPrerequisite],
    is_pm: bool,
) -> Result<()> {
    for (index, prereq) in activ_prereqs.iter().enumerate() {
        trace!("Writing activation prerequisite {}", index);
        match prereq {
            ActivationPrerequisite::Animation(prereq) => write_activ_prereq_anim(write, prereq)?,
            ActivationPrerequisite::Object(prereq) => {
                write_activ_prereq_object(write, prereq, is_pm)?
            }
            ActivationPrerequisite::Parent(prereq) => {
                write_activ_prereq_parent(write, prereq, is_pm)?
            }
        }
    }
    Ok(())
}

fn write_activ_prereq_hdr(
    write: &mut CountingWriter<impl Write>,
    required: bool,
    prereq_type: ActivPrereqType,
) -> Result<()> {
    let header = ActivPrereqHeaderC {
        opt: (!required).into(),
        ty: prereq_type.maybe(),
    };
    write.write_struct(&header)?;
    Ok(())
}

fn write_activ_prereq_anim(
    write: &mut CountingWriter<impl Write>,
    prereq: &PrerequisiteAnimation,
) -> Result<()> {
    write_activ_prereq_hdr(write, prereq.required, ActivPrereqType::Animation)?;

    let prereq = ActivPrereqAnimC {
        name: Ascii::from_str_padded(&prereq.name),
        zero32: 0,
        zero36: 0,
    };
    write.write_struct(&prereq)?;
    Ok(())
}

fn write_activ_prereq_object(
    write: &mut CountingWriter<impl Write>,
    prereq: &PrerequisiteObject,
    is_pm: bool,
) -> Result<()> {
    write_activ_prereq_hdr(write, prereq.required, ActivPrereqType::Object)?;

    let name = Ascii::from_str_padded(&prereq.name);

    let active = if prereq.active {
        match is_pm {
            true => 3,
            false => 1,
        }
    } else {
        0
    };

    let prereq = ActivPrereqObjC {
        active,
        name,
        ptr: Ptr(prereq.ptr),
    };
    write.write_struct(&prereq)?;
    Ok(())
}

fn write_activ_prereq_parent(
    write: &mut CountingWriter<impl Write>,
    prereq: &PrerequisiteParent,
    is_pm: bool,
) -> Result<()> {
    write_activ_prereq_hdr(write, prereq.required, ActivPrereqType::Parent)?;

    let name = Ascii::from_str_padded(&prereq.name);

    let active = if prereq.active {
        match is_pm {
            true => 2,
            false => 1,
        }
    } else {
        0
    };

    let prereq = ActivPrereqObjC {
        active,
        name,
        ptr: Ptr(prereq.ptr),
    };
    write.write_struct(&prereq)?;
    Ok(())
}
