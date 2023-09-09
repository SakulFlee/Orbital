use crate::{app::app_world::renderable::Renderable, engine::vertex::Vertex};

pub struct Triangle;

impl Triangle {
    const VERTICES: [Vertex; 3] = [
        Vertex {
            position: [0.0, 0.5, 0.0],
            color: [1.0, 0.0, 0.0],
        },
        Vertex {
            position: [-0.5, -0.5, 0.0],
            color: [0.0, 1.0, 0.0],
        },
        Vertex {
            position: [0.5, -0.5, 0.0],
            color: [0.0, 0.0, 1.0],
        },
    ];
}

impl Renderable for Triangle {
    fn vertices(&self) -> &[Vertex] {
        &Self::VERTICES
    }

    fn do_render(&self) -> bool {
        true
    }
}
