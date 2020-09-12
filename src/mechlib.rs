use crate::assert::AssertionError;
use crate::io_ext::{CountingReader, WriteHelper};
use crate::materials::{read_material, write_material, RawMaterial, TexturedMaterial};
use crate::mesh::{read_mesh_data, read_mesh_info, write_mesh_data, write_mesh_info, Mesh};
use crate::nodes::{
    read_node_data, read_node_info_mechlib, write_node_data, write_node_info, Node as BaseNode,
    WrappedNode,
};
use crate::{assert_that, Result};
use ::serde::{Deserialize, Serialize};
use std::io::{Read, Write};

pub type Material = crate::materials::Material;

const VERSION: u32 = 27;
const FORMAT: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
struct Node(BaseNode<Node>);

#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
    root: Node,
    meshes: Vec<Mesh>,
}

pub fn read_version<R>(read: &mut CountingReader<R>) -> Result<()>
where
    R: Read,
{
    let version = read.read_u32()?;
    assert_that!("version", version == VERSION, read.prev)?;
    read.assert_end()
}

pub fn read_format<R>(read: &mut CountingReader<R>) -> Result<()>
where
    R: Read,
{
    let format = read.read_u32()?;
    assert_that!("format", format == FORMAT, read.prev)?;
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

pub fn read_materials<R>(read: &mut CountingReader<R>) -> Result<Vec<Material>>
where
    R: Read,
{
    let count = read.read_u32()?;
    let materials = (0..count)
        .map(|_| {
            let material = read_material(read)?;
            Ok(match material {
                RawMaterial::Textured(mat) => {
                    // mechlib materials cannot have cycled textures
                    assert_that!("cycle ptr", mat.cycle_ptr == None, read.prev + 36)?;
                    // mechlib materials store the texture name immediately after
                    let texture = read.read_string()?;
                    Material::Textured(TexturedMaterial {
                        texture,
                        pointer: mat.pointer,
                        cycle: None,
                        unk32: mat.unk32,
                        flag: mat.flag,
                    })
                }
                RawMaterial::Colored(mat) => Material::Colored(mat),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    read.assert_end()?;
    Ok(materials)
}

pub fn write_materials<W>(write: &mut W, materials: &[Material]) -> Result<()>
where
    W: Write,
{
    write.write_u32(materials.len() as u32)?;
    for material in materials {
        write_material(write, material, None)?;
        if let Material::Textured(textured) = material {
            if textured.cycle.is_some() {
                panic!("mechlib materials cannot have cycled textures");
            }
            write.write_string(&textured.texture)?;
        }
    }
    Ok(())
}

fn read_node_and_mesh<R>(read: &mut CountingReader<R>, meshes: &mut Vec<Mesh>) -> Result<Node>
where
    R: Read,
{
    let variant = read_node_info_mechlib(read)?;
    match read_node_data(read, variant)? {
        WrappedNode::Object3d(wrapped) => {
            let mut object3d = wrapped.wrapped;
            if object3d.mesh_index != 0 {
                let wrapped_mesh = read_mesh_info(read)?;
                let mesh = read_mesh_data(read, wrapped_mesh)?;
                meshes.push(mesh);
            }

            // we have to apply this, so data is written out correctly again, even if
            // the mechlib data doesn't read/write parents
            object3d.parent = if wrapped.has_parent { Some(0) } else { None };

            object3d.children = (0..wrapped.children_count)
                .map(|_| read_node_and_mesh(read, meshes))
                .collect::<Result<Vec<_>>>()?;

            Ok(Node(BaseNode::Object3d(object3d)))
        }
        _ => Err(AssertionError("Expected only Object3d nodes in mechlib".to_owned()).into()),
    }
}

pub fn read_model<R>(read: &mut CountingReader<R>) -> Result<Model>
where
    R: Read,
{
    let mut meshes = vec![];
    let root = read_node_and_mesh(read, &mut meshes)?;
    read.assert_end()?;
    Ok(Model { root, meshes })
}

fn write_node_and_mesh<W>(
    write: &mut W,
    node: &Node,
    meshes: &[Mesh],
    mesh_index: &mut usize,
) -> Result<()>
where
    W: Write,
{
    write_node_info(write, &node.0)?;
    write_node_data(write, &node.0)?;
    match &node.0 {
        BaseNode::Object3d(object3d) => {
            if object3d.mesh_index != 0 {
                let mesh = &meshes[*mesh_index];
                write_mesh_info(write, mesh)?;
                write_mesh_data(write, mesh)?;
                *mesh_index += 1;
            }

            for child in &object3d.children {
                write_node_and_mesh(write, child, meshes, mesh_index)?;
            }
            Ok(())
        }
        _ => Err(AssertionError("Expected only Object3d nodes in mechlib".to_owned()).into()),
    }
}

pub fn write_model<W>(write: &mut W, model: &Model) -> Result<()>
where
    W: Write,
{
    let mut mesh_index = 0;
    write_node_and_mesh(write, &model.root, &model.meshes, &mut mesh_index)
}
