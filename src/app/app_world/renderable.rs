use crate::engine::vertex::Vertex;

pub trait Renderable {
    fn vertices(&self) -> &[Vertex];

    fn do_render(&self) -> bool;
}
