use mech3ax_api_types::nodes::mw::{Camera, Display, Empty, Light, Lod, Object3d, Window, World};

pub struct WrapperRc<T> {
    pub wrapped: T,
    pub has_parent: bool,
    pub children_count: u32,
}

pub enum WrappedNodeRc {
    Camera(Camera),
    Display(Display),
    Empty(Empty),
    Light(Light),
    Lod(WrapperRc<Lod>),
    Object3d(WrapperRc<Object3d>),
    Window(Window),
    World(WrapperRc<World>),
}
