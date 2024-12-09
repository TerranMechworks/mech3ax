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

        // TODO
        let (unk, bytes) = object.unk.split_last_chunk().unwrap_or((&[], &[0, 0]));
        let flags = u16::from_le_bytes(*bytes);
        let unk = Bytes::from_slice(unk);

        let object_c = ObjectRefC {
            name,
            zero32: 0,
            ptr: Ptr::INVALID,
            flags,
            root_idx: 0,
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

        // TODO
        let flags = node.ptr as _;

        let name = Ascii::from_str_node_name(&node.name);
        let node_c = NodeRefC {
            flags,
            root_idx: 0,
            name,
            zero36: 0,
            ptr: Ptr::INVALID,
        };
        write.write_struct(&node_c)?;
    }
    Ok(())
}
