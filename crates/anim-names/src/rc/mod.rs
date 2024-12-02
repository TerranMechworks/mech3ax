mod name {
    include!(concat!(env!("OUT_DIR"), "/rc_anim_names.rs"));
}

mod root {
    include!(concat!(env!("OUT_DIR"), "/rc_anim_root_names.rs"));
}

mod list {
    include!(concat!(env!("OUT_DIR"), "/rc_anim_list.rs"));
}

use crate::{fwd, rev};

fwd!(anim_name_fwd, 32, name::INDEX, name::TABLE);
rev!(anim_name_rev, 32, name::INDEX, name::TABLE);

fwd!(anim_root_name_fwd, 32, root::INDEX, root::TABLE);
rev!(anim_root_name_rev, 32, root::INDEX, root::TABLE);

fwd!(anim_list_fwd, 80, list::INDEX, list::TABLE);
rev!(anim_list_rev, 80, list::INDEX, list::TABLE);

#[cfg(test)]
mod tests;
