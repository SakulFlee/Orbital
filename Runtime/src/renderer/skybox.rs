use std::error::Error;

use wgpu::{
    CommandBuffer, CommandEncoderDescriptor, Device, LoadOp, Operations, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, StoreOp, TextureFormat, TextureView,
};

use crate::{
    resources::{
        Camera, MaterialShader, WorldEnvironment, WorldEnvironmentDescriptor,
    },
    world::World,
};

use super::{system::RenderSystem, RenderEvent};

#[derive(Debug)]
pub struct SkyBoxRenderer {
    surface_texture_format: TextureFormat,
    world_environment_material_shader: Option<MaterialShader>,
}

impl SkyBoxRenderer {
    pub fn new(surface_texture_format: TextureFormat, device: &Device, queue: &Queue) -> Self {
        Self {
            surface_texture_format,
            world_environment_material_shader: None,
        }
    }

    pub fn change_world_environment(
        &mut self,
        descriptor: &WorldEnvironmentDescriptor,
        device: &Device,
        queue: &Queue,
    ) -> Result<(), Box<dyn Error>> {
        match WorldEnvironment::from_descriptor(descriptor, device, queue) {
            Ok(x) => {
                let material_descriptor = x.into_material_shader_descriptor(device, queue);
                match MaterialShader::from_descriptor(
                    &material_descriptor,
                    Some(self.surface_texture_format),
                    device,
                    queue,
                ) {
                    Ok(x) => {
                        self.world_environment_material_shader = Some(x);
                    }
                    Err(e) => {
                        self.world_environment_material_shader = None;
                        return Err(e.into());
                    }
                }
            }
            Err(e) => {
                self.world_environment_material_shader = None;
                return Err(e.into());
            }
        }

        Ok(())
    }
}

impl RenderSystem for SkyBoxRenderer {
    async fn change_surface_texture_format(
        &mut self,
        surface_texture_format: TextureFormat,
        _device: &Device,
        _queue: &Queue,
    ) {
        self.surface_texture_format = surface_texture_format;
    }

    async fn change_resolution(
        &mut self,
        _resolution: cgmath::Vector2<u32>,
        _device: &Device,
        _queue: &Queue,
    ) {
        // Intentionally empty! :)
        // Nothing to do here ...
    }

    async fn update(&mut self, world: &World, events: Option<Vec<RenderEvent>>) {
        todo!()
    }

    async fn render(
        &mut self,
        target_view: &TextureView,
        camera: &Camera,
        device: &Device,
        _queue: &Queue,
    ) -> Option<CommandBuffer> {
        let world_environment_material = if let Some(x) = &self.world_environment_material_shader {
            x
        } else {
            return None;
        };

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("CommandEncoder::Skybox"),
        });

        // Scope to drop render pass once done
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("RenderPass::SkyBox"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load, // TODO: Possibly should be discard?
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(world_environment_material.pipeline());

            render_pass.set_bind_group(0, world_environment_material.bind_group(), &[]);

            render_pass.set_bind_group(1, camera.camera_bind_group(), &[]);

            render_pass.draw(0..3, 0..1);
        }

        Some(encoder.finish())
    }
}
