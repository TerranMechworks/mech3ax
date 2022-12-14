use mech3ax_api_types::nodes::pm::*;

pub struct WrapperPm<T> {
    pub wrapped: T,
    pub has_parent: bool,
    pub children_count: u16,
}

pub enum WrappedNodePm {
    World(WrapperPm<World>),
    Window(Window),
    Camera(Camera),
    Display(Display),
    Light(Light),
    Lod(WrapperPm<Lod>),
    Object3d(WrapperPm<Object3d>),
}
