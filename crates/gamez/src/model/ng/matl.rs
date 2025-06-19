use super::{MaterialRefC, PolygonPmC};
use mech3ax_api_types::gamez::materials::Material;
use mech3ax_api_types::gamez::model::Model;
use mech3ax_types::{u32_to_usize, AsBytes as _, Ptr};

pub(crate) fn make_material_refs(
    materials: &[Material],
    model: &Model,
    is_mechlib: bool,
) -> Vec<MaterialRefC> {
    let mut cycled_infos: Vec<MaterialRefC> = Vec::new();
    let mut normal_infos: Vec<MaterialRefC> = Vec::new();

    for (polygon, index) in model.polygons.iter().zip(0u32..) {
        let polygon_ptr = if is_mechlib {
            Ptr::NULL
        } else {
            Ptr(model.polygons_ptr.wrapping_add(index * PolygonPmC::SIZE))
        };

        if let Some(matl) = polygon.materials.first() {
            let is_cycled = materials
                .get(u32_to_usize(matl.material_index))
                .map(Material::is_cycled)
                .unwrap_or_default();

            if is_cycled {
                // cycled infos are not de-duplicated
                cycled_infos.push(MaterialRefC {
                    material_index: matl.material_index,
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
                            material_index: matl.material_index,
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
