use crate::{app::app_world::renderable::Renderable, engine::vertex::Vertex};

pub struct Triangle;

impl Triangle {
    const VERTICES: &[Vertex] = &[
        // A
        Vertex {
            position: [0.0, 0.35, 0.0],
            color: [1.0, 0.0, 0.0],
        },
        // B
        Vertex {
            position: [-0.35, -0.35, 0.0],
            color: [0.0, 1.0, 0.0],
        },
        // C
        Vertex {
            position: [0.35, -0.35, 0.0],
            color: [0.0, 0.0, 1.0],
        },
    ];

    const INDICES: &[u16] = &[0, 1, 2];
}

impl Renderable for Triangle {
    fn vertices(&self) -> &[Vertex] {
        &Self::VERTICES
    }

    fn indices(&self) -> &[u16] {
        &Self::INDICES
    }
}
