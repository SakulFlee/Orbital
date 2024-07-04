use cgmath::Point3;
use wgpu::{Device, Queue, TextureFormat, TextureView};

use crate::resources::{descriptors::CameraDescriptor, realizations::Model};

use super::{Renderer, StandardRenderer};

pub struct TestRenderer {
    standard: StandardRenderer,
    camera_change: Option<CameraDescriptor>,
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

        Self {
            standard,
            camera_change: None,
        }
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

    fn update(&mut self, delta_time: f64) {
        self.standard.update(delta_time);

        // Queue camera change
        unsafe {
            static mut INCREMENT: bool = true;
            let mut camera_change = *self.standard.camera_descriptor();

            if INCREMENT {
                camera_change.position.x += 1.0 * delta_time as f32;
            } else {
                camera_change.position.x -= 1.0 * delta_time as f32;
            }

            if camera_change.position.x >= 5.0 {
                INCREMENT = false;
            }
            if camera_change.position.x <= 1.5 {
                INCREMENT = true;
            }

            self.camera_change = Some(camera_change);
        }
    }

    fn render(
        &mut self,
        target_view: &TextureView,
        device: &Device,
        queue: &Queue,
        models: &[&Model],
    ) {
        // If there is a camera change queued, update the camera before rendering.
        if let Some(camera_change) = self.camera_change {
            self.standard.change_camera(camera_change, device, queue);
        }

        // Render
        self.standard.render(target_view, device, queue, models);
    }
}
