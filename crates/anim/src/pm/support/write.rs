use super::{NodeRefC, ObjectRefC};
use log::trace;
use mech3ax_api_types::anim::{NodeRef, ObjectRef};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::Result;
use mech3ax_types::{Ascii, Bytes, EnumerateEx as _, Ptr};
use std::io::Write;

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

        let name = Ascii::from_str_node_name(&object.name);
        let unk = Bytes::from_slice(&object.unk);
        let object_c = ObjectRefC {
            name,
            zero32: 0,
            mone36: u32::MAX,
            unk,
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

        let name = Ascii::from_str_node_name(&node.name);
        let node_c = NodeRefC {
            zero00: node.ptr,
            name,
            zero36: 0,
            ptr: Ptr::INVALID,
        };
        write.write_struct(&node_c)?;
    }
    Ok(())
}
