use super::{EventMw, EventPm, EventRc};
use crate::types::{index, AnimDefLookup as _, Idx16, INPUT_NODE_NAME};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::{
    CallAnimation, CallAnimationAtNode, CallAnimationParameters, CallAnimationWithNode,
};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::Vec3;
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{bitflags, impl_as_bytes, AsBytes as _, Ascii, Maybe};
use std::io::{Read, Write};

bitflags! {
    struct CallAnimationFlags: u16 {
        const AT_NODE = 1 << 0;             // 0x01
        // AT_NODE/WITH_NODE has relative position
        const POSITION = 1 << 1;            // 0x02
        // AT_NODE has additional translate
        const TRANSLATE = 1 << 2;           // 0x04
        const WITH_NODE = 1 << 3;           // 0x08
        // not in RC?
        const WAIT_FOR_COMPLETION = 1 << 4; // 0x10
    }
}

type Flags = Maybe<u16, CallAnimationFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct CallAnimationC {
    anim_name: Ascii<32>,       // 00
    operand_index: Idx16,       // 32
    flags: Flags,               // 34
    anim_index: i16,            // 36
    wait_for_completion: Idx16, // 38
    node_index: Idx16,          // 40
    pad42: u16,                 // 42
    position: Vec3,             // 44
    translate: Vec3,            // 56
}
impl_as_bytes!(CallAnimationC, 68);

fn read_call_animation(
    read: &mut CountingReader<impl Read>,
    anim_def: &AnimDef,
    size: u32,
    is_rc: bool,
) -> Result<CallAnimation> {
    assert_that!(
        "call animation size",
        size == CallAnimationC::SIZE,
        read.offset
    )?;
    let call_anim: CallAnimationC = read.read_struct()?;

    let name = assert_utf8("call animation name", read.prev + 0, || {
        call_anim.anim_name.to_str_padded()
    })?;

    let operand_node = if call_anim.operand_index == index!(0) {
        None
    } else {
        Some(anim_def.node_from_index(call_anim.operand_index, read.prev + 32)?)
    };

    let flags = assert_that!("call animation flags", flags call_anim.flags, read.prev + 34)?;

    // the anim index can't be set on load...
    assert_that!(
        "call animation anim index",
        call_anim.anim_index == 0,
        read.prev + 36
    )?;

    let position = if flags.contains(CallAnimationFlags::POSITION) {
        Some(call_anim.position)
    } else {
        assert_that!(
            "call animation position",
            call_anim.position == Vec3::DEFAULT,
            read.prev + 44
        )?;
        None
    };

    let translate = if flags.contains(CallAnimationFlags::TRANSLATE) {
        Some(call_anim.translate)
    } else {
        assert_that!(
            "call animation translate",
            call_anim.translate == Vec3::DEFAULT,
            read.prev + 56
        )?;
        None
    };

    let has_with_node = flags.contains(CallAnimationFlags::WITH_NODE);
    let parameters = if flags.contains(CallAnimationFlags::AT_NODE) {
        // should be mutually exclusive with WITH_NODE
        assert_that!(
            "call animation is with node",
            has_with_node == false,
            read.prev + 34
        )?;

        let node = if call_anim.node_index == index!(input) {
            INPUT_NODE_NAME.to_string()
        } else {
            anim_def.node_from_index(call_anim.node_index, read.prev + 40)?
        };

        Some(CallAnimationParameters::AtNode(CallAnimationAtNode {
            node,
            position,
            translate,
        }))
    } else if has_with_node {
        let node = if call_anim.node_index == index!(input) {
            INPUT_NODE_NAME.to_string()
        } else {
            anim_def.node_from_index(call_anim.node_index, read.prev + 40)?
        };

        let has_translate = translate.is_some();
        assert_that!(
            "call animation has translate",
            has_translate == false,
            read.prev + 34
        )?;

        Some(CallAnimationParameters::WithNode(CallAnimationWithNode {
            node,
            position,
        }))
    } else {
        // the node index isn't used
        assert_that!(
            "call animation node index",
            call_anim.node_index == index!(0),
            read.prev + 40
        )?;

        let has_position = position.is_some();
        assert_that!(
            "call animation has position",
            has_position == false,
            read.prev + 34
        )?;
        let has_translate = translate.is_some();
        assert_that!(
            "call animation has translate",
            has_translate == false,
            read.prev + 34
        )?;

        None
    };

    // RC doesn't seem to have WAIT_FOR_COMPLETION
    let has_wait_for = if is_rc {
        call_anim.wait_for_completion > -1
    } else {
        flags.contains(CallAnimationFlags::WAIT_FOR_COMPLETION)
    };

    let wait_for_completion = if has_wait_for {
        let index = anim_def.anim_ref_from_index(call_anim.wait_for_completion, read.prev + 38)?;
        Some(index)
    } else {
        assert_that!(
            "call animation wait for",
            call_anim.wait_for_completion == index!(-1),
            read.prev + 38
        )?;
        None
    };

    assert_that!(
        "call animation node field 42",
        call_anim.pad42 == 0,
        read.prev + 42
    )?;

    Ok(CallAnimation {
        name,
        operand_node,
        wait_for_completion,
        parameters,
    })
}

fn write_call_animation(
    call_anim: &CallAnimation,
    write: &mut CountingWriter<impl Write>,
    anim_def: &AnimDef,
    is_rc: bool,
) -> Result<()> {
    let anim_name = Ascii::from_str_padded(&call_anim.name);
    let operand_index = match &call_anim.operand_node {
        Some(name) => anim_def.node_to_index(name)?,
        None => index!(0),
    };

    let mut flags = CallAnimationFlags::empty();

    let wait_for_completion = match &call_anim.wait_for_completion {
        Some(index) => {
            // RC doesn't seem to have WAIT_FOR_COMPLETION
            if !is_rc {
                flags |= CallAnimationFlags::WAIT_FOR_COMPLETION;
            }
            anim_def.anim_ref_to_index(*index)?
        }
        None => index!(-1),
    };

    let mut position = Vec3::DEFAULT;
    let mut translate = Vec3::DEFAULT;

    let node_index = match &call_anim.parameters {
        Some(CallAnimationParameters::WithNode(CallAnimationWithNode {
            node,
            position: maybe_p,
        })) => {
            flags |= CallAnimationFlags::WITH_NODE;

            if let Some(p) = maybe_p {
                flags |= CallAnimationFlags::POSITION;
                position = *p;
            };

            if node == INPUT_NODE_NAME {
                index!(input)
            } else {
                anim_def.node_to_index(node)?
            }
        }
        Some(CallAnimationParameters::AtNode(CallAnimationAtNode {
            node,
            position: maybe_p,
            translate: maybe_t,
        })) => {
            flags |= CallAnimationFlags::AT_NODE;

            if let Some(p) = maybe_p {
                flags |= CallAnimationFlags::POSITION;
                position = *p;
            };
            if let Some(t) = maybe_t {
                flags |= CallAnimationFlags::TRANSLATE;
                translate = *t;
            }

            if node == INPUT_NODE_NAME {
                index!(input)
            } else {
                anim_def.node_to_index(node)?
            }
        }
        None => index!(0),
    };

    let call_anim = CallAnimationC {
        anim_name,
        operand_index,
        flags: flags.maybe(),
        anim_index: 0,
        wait_for_completion,
        node_index,
        pad42: 0,
        position,
        translate,
    };
    write.write_struct(&call_anim)?;
    Ok(())
}

impl EventMw for CallAnimation {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(CallAnimationC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        read_call_animation(read, anim_def, size, false)
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        write_call_animation(self, write, anim_def, false)
    }
}

impl EventPm for CallAnimation {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(CallAnimationC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        read_call_animation(read, anim_def, size, false)
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        write_call_animation(self, write, anim_def, false)
    }
}

impl EventRc for CallAnimation {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(CallAnimationC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        read_call_animation(read, anim_def, size, true)
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        write_call_animation(self, write, anim_def, true)
    }
}
