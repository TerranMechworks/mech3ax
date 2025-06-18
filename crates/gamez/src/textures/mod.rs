pub(crate) mod mw;
pub(crate) mod ng;
pub(crate) mod rc;

use mech3ax_types::{primitive_enum, Maybe};

primitive_enum! {
    enum TextureState: u32 {
        Used = 2,
    }
}

type State = Maybe<u32, TextureState>;
