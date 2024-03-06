// use crate::{app::AppObject, ModelVertex};

// pub struct Square;

// impl Square {
//     const VERTICES: &[ModelVertex] = &[
//         // A
//         ModelVertex {
//             position: [-0.5, -0.5, 0.0],
//             tex_coords: [1.0, 1.0],
//             normal: [0.0, 0.0, 1.0],
//         },
//         // B
//         ModelVertex {
//             position: [0.5, -0.5, 0.0],
//             tex_coords: [0.0, 1.0],
//             normal: [0.0, 0.0, 1.0],
//         },
//         // C
//         ModelVertex {
//             position: [0.5, 0.5, 0.0],
//             tex_coords: [0.0, 0.0],
//             normal: [0.0, 0.0, 1.0],
//         },
//         // D
//         ModelVertex {
//             position: [-0.5, 0.5, 0.0],
//             tex_coords: [1.0, 0.0],
//             normal: [0.0, 0.0, 1.0],
//         },
//     ];

//     const INDICES: &[u16] = &[0, 1, 3, 1, 2, 3];
// }

// impl AppObject for Square {
//     fn vertices(&self) -> &[ModelVertex] {
//         &Self::VERTICES
//     }

//     fn indices(&self) -> &[u16] {
//         &Self::INDICES
//     }

//     fn do_render(&self) -> bool {
//         true
//     }
// }
