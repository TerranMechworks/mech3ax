use mech3ax_api_types::nodes::pm::{Lod, Object3d};

pub struct WrapperPm<T> {
    pub wrapped: T,
    pub has_parent: bool,
    pub children_count: u16,
}

pub enum WrappedNodePm {
    Lod(WrapperPm<Lod>),
    Object3d(WrapperPm<Object3d>),
}
