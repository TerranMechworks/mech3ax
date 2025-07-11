use super::{MaterialRefC, PolygonPmC};
use mech3ax_api_types::gamez::materials::Material;
use mech3ax_api_types::gamez::model::Model;
use mech3ax_types::{AsBytes as _, Ptr};

pub(crate) fn make_material_refs(materials: &[Material], model: &Model) -> Vec<MaterialRefC> {
    let mut cycled_infos: Vec<MaterialRefC> = Vec::new();
    let mut normal_infos: Vec<MaterialRefC> = Vec::new();

    for (polygon, index) in model.polygons.iter().zip(0u32..) {
        let polygon_ptr = Ptr(model.polygons_ptr.wrapping_add(index * PolygonPmC::SIZE));

        if let Some(matl) = polygon.materials.first() {
            let is_cycled = materials
                .get(matl.material_index.to_usize())
                .map(Material::is_cycled)
                .unwrap_or_default();

            if is_cycled {
                // cycled infos are not de-duplicated
                cycled_infos.push(MaterialRefC {
                    material_index: matl.material_index.maybe(),
                    usage_count: 1,
                    polygon_ptr,
                });
            } else {
                let existing = normal_infos
                    .iter_mut()
                    .find(|mi| mi.material_index == matl.material_index);

                match existing {
                    Some(mi) => mi.usage_count += 1,
                    None => {
                        normal_infos.push(MaterialRefC {
                            material_index: matl.material_index.maybe(),
                            usage_count: 1,
                            polygon_ptr,
                        });
                    }
                }
            }
        }
    }

    if cycled_infos.is_empty() {
        normal_infos
    } else {
        // cycled ones go first
        cycled_infos.append(&mut normal_infos);
        cycled_infos
    }
}

pub(crate) fn make_material_refs_mechlib(model: &Model) -> Vec<MaterialRefC> {
    // mechlib materials cannot be cycled and the pointers are zero, so the
    // general case can be simplified. this also helps us, as the materials
    // aren't know when writing the models; the materials are written at the end
    // of the ZBD.
    let mut normal_infos: Vec<MaterialRefC> = Vec::new();

    for polygon in model.polygons.iter() {
        if let Some(matl) = polygon.materials.first() {
            let existing = normal_infos
                .iter_mut()
                .find(|mi| mi.material_index == matl.material_index);

            match existing {
                Some(mi) => mi.usage_count += 1,
                None => {
                    normal_infos.push(MaterialRefC {
                        material_index: matl.material_index.maybe(),
                        usage_count: 1,
                        polygon_ptr: Ptr::NULL,
                    });
                }
            }
        }
    }

    normal_infos
}
