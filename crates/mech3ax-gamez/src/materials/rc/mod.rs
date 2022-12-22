mod read_multi;
mod read_single;
mod write_multi;
mod write_single;

use super::{assert_material_info, find_texture_index_by_name, MaterialC, MaterialInfoC};
use mech3ax_api_types::ReprSize as _;

bitflags::bitflags! {
    struct MaterialFlags: u8 {
        const TEXTURED = 1 << 0;
    }
}

macro_rules! material_array_size {
    () => {
        5000
    };
}
pub(crate) use material_array_size;

pub fn size_materials() -> u32 {
    // Cast safety: truncation simply leads to incorrect size (TODO?)
    MaterialInfoC::SIZE + (MaterialC::SIZE + 2 + 2) * material_array_size!()
}

pub use read_multi::read_materials;
pub use write_multi::write_materials;
