mod read;
mod write;

mod size {
    pub(crate) fn size() -> u32 {
        use mech3ax_types::AsBytes as _;
        mech3ax_api_types::gamez::nodes::Display::SIZE
    }
}

pub(crate) mod rc {
    pub(crate) use super::read::read;
    pub(crate) use super::size::size;
    pub(crate) use super::write::write;
}
