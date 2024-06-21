use cgmath::Point3;
use wgpu::{Device, Queue, TextureFormat, TextureView};

use crate::resources::{descriptors::CameraDescriptor, realizations::Model};

use super::{Renderer, StandardRenderer};

pub struct TestRenderer {
    standard: StandardRenderer,
}

impl TestRenderer {
    pub fn update(&mut self, device: &Device, queue: &Queue) {
        unsafe {
            static mut INCREMENT: bool = true;
            let mut x = *self.standard.camera_descriptor();

            if INCREMENT {
                x.position.x += 0.025;
            } else {
                x.position.x -= 0.025;
            }

            if x.position.x >= 5.0 {
                INCREMENT = false;
            }
            if x.position.x <= 1.5 {
                INCREMENT = true;
            }

            self.standard.change_camera(x, device, queue);
        }
    }
}

impl Renderer for TestRenderer {
    fn new(
        surface_texture_format: TextureFormat,
        resolution: cgmath::Vector2<u32>,
        device: &Device,
        queue: &Queue,
    ) -> Self {
        let mut standard = StandardRenderer::new(surface_texture_format, resolution, device, queue);

        standard.change_camera(
            CameraDescriptor {
                position: Point3::new(5.0, 0.0, 0.0),
                ..Default::default()
            },
            device,
            queue,
        );

        Self { standard }
    }

    fn change_surface_texture_format(
        &mut self,
        surface_texture_format: TextureFormat,
        device: &Device,
        queue: &Queue,
    ) {
        self.standard
            .change_surface_texture_format(surface_texture_format, device, queue)
    }

    fn change_resolution(
        &mut self,
        resolution: cgmath::Vector2<u32>,
        device: &Device,
        queue: &Queue,
    ) {
        self.standard.change_resolution(resolution, device, queue);
    }

    fn render(
        &mut self,
        target_view: &TextureView,
        device: &Device,
        queue: &Queue,
        models: &[Model],
    ) {
        // Update camera
        self.update(device, queue);

        // Render
        self.standard.render(target_view, device, queue, models);
    }
}
