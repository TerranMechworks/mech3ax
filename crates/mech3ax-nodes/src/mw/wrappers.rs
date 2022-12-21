use mech3ax_api_types::nodes::mw::*;
use mech3ax_api_types::nodes::{Camera, Display, Window};

pub struct WrapperMw<T> {
    pub wrapped: T,
    pub has_parent: bool,
    pub children_count: u32,
}

pub enum WrappedNodeMw {
    World(WrapperMw<World>),
    Window(Window),
    Camera(Camera),
    Display(Display),
    Empty(Empty),
    Light(Light),
    Lod(WrapperMw<Lod>),
    Object3d(WrapperMw<Object3d>),
}
