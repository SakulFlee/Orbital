// use crate::{app::app_world::renderable::Renderable, engine::vertex::Vertex};

// pub struct Pentagon;

// impl Pentagon {
//     const VERTICES: &[Vertex] = &[
//         // A
//         Vertex {
//             position: [0.0, 0.5, 0.0],
//             tex_coords: [0.5, 0.5],
//         },
//         // B
//         Vertex {
//             position: [-0.5, 0.15, 0.0],
//             tex_coords: [0.0, 1.0],
//         },
//         // C
//         Vertex {
//             position: [-0.30, -0.5, 0.0],
//             tex_coords: [0.5, 0.0],
//         },
//         // D
//         Vertex {
//             position: [0.30, -0.5, 0.0],
//             tex_coords: [0.0, 0.5],
//         },
//         // E
//         Vertex {
//             position: [0.5, 0.15, 0.0],
//             tex_coords: [1.0, 0.0],
//         },
//     ];

//     const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];
// }

// impl Renderable for Pentagon {
//     fn vertices(&self) -> &[Vertex] {
//         &Self::VERTICES
//     }

//     fn indices(&self) -> &[u16] {
//         &Self::INDICES
//     }
// }
