use crate::engine::vertex::Vertex;

pub trait Renderable {
    fn vertices(&self) -> &[Vertex];

    fn indices(&self) -> &[u16];

    fn do_render(&self) -> bool {
        true
    }
}
