mod name {
    include!(concat!(env!("OUT_DIR"), "/rc_anim_names.rs"));
}

mod root {
    include!(concat!(env!("OUT_DIR"), "/rc_anim_root_names.rs"));
}

use crate::{fwd, rev};

fwd!(anim_name_fwd, name::INDEX, name::TABLE);
rev!(anim_name_rev, name::INDEX, name::TABLE);

fwd!(anim_root_name_fwd, root::INDEX, root::TABLE);
rev!(anim_root_name_rev, root::INDEX, root::TABLE);

#[cfg(test)]
mod tests;
