pub(crate) mod mw;
pub(crate) mod pm;
pub(crate) mod rc;

use mech3ax_api_types::{Count, IndexO, IndexO32};

fn mip_index(mip_index: IndexO32, count: Count) -> Result<IndexO, String> {
    count.index_opt_i32(mip_index)
}
