use super::types::Object3d;

pub struct Object3dWrapper {
    pub wrapped: Object3d,
    pub has_parent: bool,
    pub children_count: u32,
}

pub enum WrappedNode {
    Object3d(Object3dWrapper),
}
