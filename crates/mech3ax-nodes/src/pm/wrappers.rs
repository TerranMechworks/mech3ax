use mech3ax_api_types::{LodPm, Object3dPm};

pub struct WrapperPm<T> {
    pub wrapped: T,
    pub has_parent: bool,
    pub children_count: u16,
}

pub enum WrappedNodePm {
    Lod(WrapperPm<LodPm>),
    Object3d(WrapperPm<Object3dPm>),
}
