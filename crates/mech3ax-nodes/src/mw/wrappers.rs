use mech3ax_api_types::gamez::nodes::mw::{
    Camera, Display, Empty, Light, Lod, Object3d, Window, World,
};

pub struct WrapperMw<T> {
    pub wrapped: T,
    pub has_parent: bool,
    pub children_count: u32,
}

pub enum WrappedNodeMw {
    Camera(Camera),
    Display(Display),
    Empty(Empty),
    Light(Light),
    Lod(WrapperMw<Lod>),
    Object3d(WrapperMw<Object3d>),
    Window(Window),
    World(WrapperMw<World>),
}
