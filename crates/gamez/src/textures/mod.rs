pub(crate) mod mw;
pub(crate) mod pm;
pub(crate) mod rc;

use mech3ax_api_types::{Count, IndexO, IndexO32};

fn mip_index(mip_index: IndexO32, count: Count) -> Result<IndexO, String> {
    count.index_opt_i32(mip_index)
}

fn mip_index_ok(mip_index: IndexO, count: Count) -> Result<IndexO32, String> {
    if mip_index.to_i16() < count.to_i16() {
        Ok(mip_index.maybe())
    } else {
        Err(format!(
            "Invalid texture mip index: expected {} < {}",
            mip_index, count
        ))
    }
}
