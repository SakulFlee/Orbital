use crate::{engine::vertex::Vertex, app::AppObject};

pub struct Square;

impl Square {
    const VERTICES: &[Vertex] = &[
        // A
        Vertex {
            position: [-0.5, -0.5, 0.0],
            tex_coords: [1.0, 1.0],
        },
        // B
        Vertex {
            position: [0.5, -0.5, 0.0],
            tex_coords: [0.0, 1.0],
        },
        // C
        Vertex {
            position: [0.5, 0.5, 0.0],
            tex_coords: [0.0, 0.0],
        },
        // D
        Vertex {
            position: [-0.5, 0.5, 0.0],
            tex_coords: [1.0, 0.0],
        },
    ];

    const INDICES: &[u16] = &[0, 1, 3, 1, 2, 3];
}

impl AppObject for Square {
    fn vertices(&self) -> &[Vertex] {
        &Self::VERTICES
    }

    fn indices(&self) -> &[u16] {
        &Self::INDICES
    }

    fn do_render(&self) -> bool {
        true
    }
}
