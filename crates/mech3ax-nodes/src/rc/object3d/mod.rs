mod data;
mod info;

pub use data::{read, size, write};
pub use info::{assert_variants, make_variants};

/// A single node in M9 has 8 parents...
fn has_borked_parents(data_ptr: u32, parent_array_ptr: u32) -> bool {
    data_ptr == 0x0178FDA0 && parent_array_ptr == 0x017B8F90
}
