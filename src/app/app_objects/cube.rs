use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroupLayout, BufferUsages, Device, Queue,
};

use crate::{AppObject, Mesh, Model, ModelVertex};

pub struct Cube {
    model: Model,
}

impl Cube {
    pub fn new(
        device: &Device,
        queue: &Queue,
        bind_group_layout: &BindGroupLayout,
    ) -> Result<Self, String> {
        let vertices = [
            ModelVertex {
                position: (0.0, 0.0, 0.0).into(),
                tex_coords: (0.0, 1.0).into(),
                normal: (0.0, 0.0, 0.0).into(),
            },
            ModelVertex {
                position: (0.0, 0.0, 1.0).into(),
                tex_coords: (0.0, 1.0).into(),
                normal: (0.0, 0.0, 0.0).into(),
            },
            ModelVertex {
                position: (0.0, 1.0, 0.0).into(),
                tex_coords: (0.0, 1.0).into(),
                normal: (0.0, 0.0, 0.0).into(),
            },
            ModelVertex {
                position: (0.0, 1.0, 1.0).into(),
                tex_coords: (0.0, 1.0).into(),
                normal: (0.0, 0.0, 0.0).into(),
            },
            ModelVertex {
                position: (1.0, 0.0, 0.0).into(),
                tex_coords: (0.0, 1.0).into(),
                normal: (0.0, 0.0, 0.0).into(),
            },
            ModelVertex {
                position: (1.0, 0.0, 1.0).into(),
                tex_coords: (0.0, 1.0).into(),
                normal: (0.0, 0.0, 0.0).into(),
            },
            ModelVertex {
                position: (1.0, 1.0, 0.0).into(),
                tex_coords: (0.0, 1.0).into(),
                normal: (0.0, 0.0, 0.0).into(),
            },
            ModelVertex {
                position: (1.0, 1.0, 1.0).into(),
                tex_coords: (0.0, 1.0).into(),
                normal: (0.0, 0.0, 0.0).into(),
            },
        ];

        #[rustfmt::skip]
        let indices = vec![
            // 1,  7,  5,
            // 1,  3,  7,
            // 1,  4,  3,
            // 1,  2,  4,
            // 3,  8,  7,
            // 3,  4,  8,
            // 5,  7,  8,
            // 5,  8,  6,
            // 1,  5,  6,
            // 1,  6,  2,
            // 2,  6,  8,
            // 2,  8,  4,
            0, 6, 4, 
            0, 2, 6, 
            0, 3, 2, 
            0, 1, 3, 
            2, 7, 6, 
            2, 3, 7, 
            4, 6, 7, 
            4, 7, 5, 
            0, 4, 5, 
            0, 5, 1, 
            1, 5, 7, 
            1, 7, 3, 
        ];

        // let vertices = [
        //     // A
        //     ModelVertex {
        //         position: [-0.5, -0.5, 0.0],
        //         tex_coords: [1.0, 1.0],
        //         normal: [0.0, 0.0, 1.0],
        //     },
        //     // B
        //     ModelVertex {
        //         position: [0.5, -0.5, 0.0],
        //         tex_coords: [0.0, 1.0],
        //         normal: [0.0, 0.0, 1.0],
        //     },
        //     // C
        //     ModelVertex {
        //         position: [0.5, 0.5, 0.0],
        //         tex_coords: [0.0, 0.0],
        //         normal: [0.0, 0.0, 1.0],
        //     },
        //     // D
        //     ModelVertex {
        //         position: [-0.5, 0.5, 0.0],
        //         tex_coords: [1.0, 0.0],
        //         normal: [0.0, 0.0, 1.0],
        //     },
        // ];

        // #[rustfmt::skip]
        // let indices = vec![
        //     0, 1, 3, 
        //     1, 2, 3
        // ];

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Cube Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });

        // Build Index Buffer
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Cube Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });

        let model = Model {
            meshes: vec![Mesh {
                name: String::from("Cube Test"),
                vertex_buffer,
                index_buffer,
                num_elements: indices.len() as u32,
                material: 0,
                instance_range: 0..1,
            }],
            materials: vec![],
        };

        Ok(Self { model })

        // Ok(Self {
        //     model: Model::from_path("cube/cube.obj", device, queue, bind_group_layout)?,
        // })
    }
}

impl AppObject for Cube {
    fn model(&self) -> Option<&Model> {
        Some(&self.model)
    }

    fn do_render(&self) -> bool {
        true
    }
}
