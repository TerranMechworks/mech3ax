use super::types::{Camera, Display, Empty, Light, Lod, Object3d, Window, World};

pub struct Wrapper<T> {
    pub wrapped: T,
    pub has_parent: bool,
    pub children_count: u32,
}

pub enum WrappedNode<T> {
    Camera(Camera),
    Display(Display),
    Empty(Empty),
    Light(Light),
    Lod(Wrapper<Lod>),
    Object3d(Wrapper<Object3d<T>>),
    Window(Window),
    World(Wrapper<World>),
}
