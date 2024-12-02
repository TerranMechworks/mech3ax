mod name {
    include!(concat!(env!("OUT_DIR"), "/mw_anim_names_test.rs"));
}
mod root {
    include!(concat!(env!("OUT_DIR"), "/mw_anim_root_names_test.rs"));
}
mod list {
    include!(concat!(env!("OUT_DIR"), "/mw_anim_list_test.rs"));
}

use super::*;
use crate::tests::test;

test!(name, name::ALL, anim_name_fwd, anim_name_rev);
test!(root, root::ALL, anim_root_name_fwd, anim_root_name_rev);
test!(list, list::ALL, anim_list_fwd, anim_list_rev);
