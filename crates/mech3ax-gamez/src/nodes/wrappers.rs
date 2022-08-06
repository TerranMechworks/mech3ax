use mech3ax_api_types::{Camera, Display, Empty, Light, Lod, Object3d, Window, World};

pub struct Wrapper<T> {
    pub wrapped: T,
    pub has_parent: bool,
    pub children_count: u32,
}

pub enum WrappedNode {
    Camera(Camera),
    Display(Display),
    Empty(Empty),
    Light(Light),
    Lod(Wrapper<Lod>),
    Object3d(Wrapper<Object3d>),
    Window(Window),
    World(Wrapper<World>),
}
