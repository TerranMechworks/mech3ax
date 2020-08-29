use crate::io_ext::{ReadHelper, WriteHelper};
use crate::mesh::{read_mesh_data, read_mesh_info};
use crate::nodes::{read_node as read_node_wrapped, Node, WrappedNode};
use crate::{assert_that, Result};
use std::io::{Read, Write};

const VERSION: u32 = 27;
const FORMAT: u32 = 1;

pub fn read_version<R>(read: &mut R) -> Result<()>
where
    R: Read,
{
    let version = read.read_u32()?;
    assert_that!("version", version == VERSION, 0)?;
    read.assert_end()
}

pub fn read_format<R>(read: &mut R) -> Result<()>
where
    R: Read,
{
    let format = read.read_u32()?;
    assert_that!("format", format == FORMAT, 0)?;
    read.assert_end()
}

pub fn write_version<W>(write: &mut W) -> Result<()>
where
    W: Write,
{
    write.write_u32(VERSION)?;
    Ok(())
}

pub fn write_format<W>(write: &mut W) -> Result<()>
where
    W: Write,
{
    write.write_u32(FORMAT)?;
    Ok(())
}

fn read_node<R>(read: &mut R, offset: &mut u32) -> Result<Node>
where
    R: Read,
{
    match read_node_wrapped(read, offset)? {
        WrappedNode::Object3d(wrapped) => {
            let mut object3d = wrapped.wrapped;
            object3d.mesh = if wrapped.mesh_index != 0 {
                let wrapped_mesh = read_mesh_info(read, offset)?;
                let mesh = read_mesh_data(read, offset, wrapped_mesh)?;
                Some(mesh)
            } else {
                None
            };

            object3d.children = if wrapped.children_count > 0 {
                let children = (0..wrapped.children_count)
                    .into_iter()
                    .map(|_| read_node(read, offset))
                    .collect::<Result<Vec<_>>>()?;
                Some(children)
            } else {
                None
            };

            Ok(Node::Object3d(object3d))
        }
    }
}

pub fn read_model<R>(read: &mut R) -> Result<Node>
where
    R: Read,
{
    let mut offset = 0;
    let root = read_node(read, &mut offset)?;
    read.assert_end()?;
    Ok(root)
}
