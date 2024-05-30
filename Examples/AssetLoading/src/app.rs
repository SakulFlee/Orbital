use std::io::Cursor;

use akimo_runtime::{
    nalgebra::{Vector2, Vector3},
    resources::{MaterialDescriptor, MeshDescriptor, ModelDescriptor, TextureDescriptor, Vertex},
    runtime::App,
    russimp::scene::{PostProcess, Scene},
    server::RenderServer,
    wgpu::{Device, Queue, SurfaceConfiguration, TextureView},
};
use image::io::Reader;

const ASSET_BUFFER: &'static [u8] = include_bytes!("../../../Assets/Models/Cube.glb");

pub struct RenderServerTriangleApp {
    render_server: RenderServer,
}

impl App for RenderServerTriangleApp {
    fn init(config: &SurfaceConfiguration, _device: &Device, _queue: &Queue) -> Self
    where
        Self: Sized,
    {
        

        // TODO - Implement two things:
        // 1. Model::from_file/from_binary
        // Supposed to read the specified file AS A MODEL.
        // Meaning, all the meshes and materials inside should be made into
        // a SINGLE model.
        //
        // 2. Composition::from_file/from_binary
        // Supposed to turn the file into many models with materials.
        // MATERIAL CACHING WILL BE REQUIRED! As well as material referencing.
        // Each Scene::Mesh should be turned into a Model, contained
        // in the Composition.
        //
        // Furthermore, later, lights and cameras will be USED in a
        // composition, whereas in a Model they will be DROPPED!

        // let vertices = vec![
        //     // Bottom Left
        //     Vertex {
        //         position_coordinates: Vector3::new(-0.5, -0.5, 0.0),
        //         texture_coordinates: Vector2::new(0.0, 1.0),
        //     },
        //     // Bottom Right
        //     Vertex {
        //         position_coordinates: Vector3::new(0.5, -0.5, 0.0),
        //         texture_coordinates: Vector2::new(1.0, 1.0),
        //     },
        //     // Top Left
        //     Vertex {
        //         position_coordinates: Vector3::new(-0.5, 0.5, 0.0),
        //         texture_coordinates: Vector2::new(0.0, 0.0),
        //     },
        //     // Top Right
        //     Vertex {
        //         position_coordinates: Vector3::new(0.5, 0.5, 0.0),
        //         texture_coordinates: Vector2::new(1.0, 0.0),
        //     },
        // ];
        // let indices = vec![
        //     0, 1, 2, // Bottom Left Triangle
        //     1, 2, 3, // Top Right Triangle
        // ];

        // let mut render_server = RenderServer::new(config.format);

        // let texture = Reader::new(Cursor::new(include_bytes!("texture.png")))
        //     .with_guessed_format()
        //     .unwrap()
        //     .decode()
        //     .unwrap()
        //     .as_bytes()
        //     .to_vec();
        // render_server.spawn_model(ModelDescriptor {
        //     mesh_descriptor: MeshDescriptor { vertices, indices },
        //     material_descriptor: MaterialDescriptor::PBR(TextureDescriptor::StandardSRGBu8Data(
        //         texture,
        //         (800, 800),
        //     )),
        // });

        // Self { render_server }
        todo!()
    }

    fn update(&mut self) {}

    fn render(&mut self, view: &TextureView, device: &Device, queue: &Queue) {
        self.render_server.render(view, device, queue);
    }
}
