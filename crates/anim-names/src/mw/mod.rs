mod name {
    include!(concat!(env!("OUT_DIR"), "/mw_anim_names.rs"));
}

mod root {
    include!(concat!(env!("OUT_DIR"), "/mw_anim_root_names.rs"));
}

use crate::{fwd, rev};

fwd!(anim_name_fwd, 32, name::INDEX, name::TABLE);
rev!(anim_name_rev, 32, name::INDEX, name::TABLE);

fwd!(anim_root_name_fwd, 32, root::INDEX, root::TABLE);
rev!(anim_root_name_rev, 32, root::INDEX, root::TABLE);

#[cfg(test)]
mod tests;
