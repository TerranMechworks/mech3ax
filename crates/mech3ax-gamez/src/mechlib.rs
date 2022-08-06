use crate::materials::{read_material, write_material, RawMaterial};
use crate::mesh::{read_mesh_data, read_mesh_info, write_mesh_data, write_mesh_info};
use crate::nodes::{read_node_mechlib, write_object_3d_data, write_object_3d_info, Wrapper};
use mech3ax_api_types::{Material, Mesh, Model, Object3d, TexturedMaterial};
use mech3ax_common::io_ext::{CountingReader, WriteHelper};
use mech3ax_common::{assert_that, Result};
use std::io::{Read, Write};

pub const VERSION_MW: u32 = 27;
pub const VERSION_PM: u32 = 41;
pub const FORMAT: u32 = 1;

pub fn read_version<R>(read: &mut CountingReader<R>, is_pm: bool) -> Result<()>
where
    R: Read,
{
    let actual = read.read_u32()?;
    let expected = if is_pm { VERSION_PM } else { VERSION_MW };
    assert_that!("version", actual == expected, read.prev)?;
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

pub fn write_version<W>(write: &mut W, is_pm: bool) -> Result<()>
where
    W: Write,
{
    write.write_u32(if is_pm { VERSION_PM } else { VERSION_MW })?;
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

fn read_node_and_mesh<R>(
    read: &mut CountingReader<R>,
    nodes: &mut Vec<Object3d>,
    meshes: &mut Vec<Mesh>,
    mesh_ptrs: &mut Vec<i32>,
) -> Result<u32>
where
    R: Read,
{
    let Wrapper {
        wrapped: mut object3d,
        has_parent,
        children_count,
    } = read_node_mechlib(read)?;

    if object3d.mesh_index != 0 {
        let ptr_index: i32 = mesh_ptrs.len().try_into().unwrap();
        // preserve the pointer, store the new index
        mesh_ptrs.push(object3d.mesh_index);
        object3d.mesh_index = ptr_index;

        let wrapped_mesh = read_mesh_info(read)?;
        let mesh = read_mesh_data(read, wrapped_mesh)?;
        meshes.push(mesh);
    } else {
        object3d.mesh_index = -1;
    }

    // we have to apply this, so data is written out correctly again, even if
    // the mechlib data doesn't read/write parents
    object3d.parent = if has_parent { Some(0) } else { None };

    let current_index = nodes.len();
    nodes.push(object3d);

    let child_indices = (0..children_count)
        .map(|_| read_node_and_mesh(read, nodes, meshes, mesh_ptrs))
        .collect::<Result<Vec<_>>>()?;

    let object3d = &mut nodes[current_index];
    object3d.children = child_indices;

    Ok(current_index.try_into().unwrap())
}

pub fn read_model<R>(read: &mut CountingReader<R>) -> Result<Model>
where
    R: Read,
{
    let mut nodes = Vec::new();
    let mut meshes = Vec::new();
    let mut mesh_ptrs = Vec::new();
    let _root_index = read_node_and_mesh(read, &mut nodes, &mut meshes, &mut mesh_ptrs)?;
    read.assert_end()?;
    Ok(Model {
        nodes,
        meshes,
        mesh_ptrs,
    })
}

fn write_node_and_mesh<W>(
    write: &mut W,
    node_index: u32,
    nodes: &mut [Object3d],
    meshes: &[Mesh],
    mesh_ptrs: &[i32],
) -> Result<()>
where
    W: Write,
{
    let object3d = &mut nodes[node_index as usize];

    // preserve mesh_index
    // if the mesh_index isn't -1, then we need to restore the correct pointer
    // before the node is written out
    let restore_index = if object3d.mesh_index > -1 {
        let index = object3d.mesh_index as usize;
        object3d.mesh_index = mesh_ptrs[index];
        Some(index)
    } else {
        object3d.mesh_index = 0;
        None
    };

    write_object_3d_info(write, &object3d)?;
    write_object_3d_data(write, &object3d)?;

    // if mesh_index isn't -1, then we need to write out the mesh, too
    if let Some(index) = restore_index {
        let mesh = &meshes[index];
        write_mesh_info(write, mesh)?;
        write_mesh_data(write, mesh)?;
    }

    let child_indices = object3d.children.clone();
    for child_index in child_indices.into_iter() {
        write_node_and_mesh(write, child_index, nodes, meshes, mesh_ptrs)?;
    }
    Ok(())
}

pub fn write_model<W>(write: &mut W, model: &mut Model) -> Result<()>
where
    W: Write,
{
    write_node_and_mesh(write, 0, &mut model.nodes, &model.meshes, &model.mesh_ptrs)
}
