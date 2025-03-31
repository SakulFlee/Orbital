use std::cell::RefCell;

use async_trait::async_trait;
use cgmath::Vector2;
use hashbrown::HashMap;
use wgpu::{
    CommandEncoder, CommandEncoderDescriptor, Device, IndexFormat, LoadOp, Operations, Queue,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, StoreOp,
    TextureFormat, TextureView,
};

use crate::cache_state::CacheState;
use crate::log::error;
use crate::resources::realizations::{Material, Model};
use crate::resources::{
    descriptors::TextureDescriptor,
    realizations::{Pipeline, Texture},
};
use crate::world::{Change, ChangeList, ChangeType, World};

use super::Renderer;

pub struct CachingDirectRenderer {
    app_name: String,
    surface_format: TextureFormat,
    depth_texture: Texture,
    world_environment: Option<Material>,
    world_environment_pipeline: Option<Pipeline>,
    model_cache: HashMap<String, Model>,
    cache_state: CacheState,
}

impl CachingDirectRenderer {
    fn render_skybox(
        &self,
        world: &World,
        encoder: &mut CommandEncoder,
        target_view: &TextureView,
    ) {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: target_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(
            self.world_environment_pipeline
                .as_ref()
                .unwrap()
                .render_pipeline(),
        );
        render_pass.set_bind_group(
            0,
            self.world_environment.as_ref().unwrap().bind_group(),
            &[],
        );
        render_pass.set_bind_group(1, world.active_camera().camera_bind_group(), &[]);
        render_pass.draw(0..3, 0..1);
    }

    fn render_models(
        &self,
        world: &World,
        encoder: &mut CommandEncoder,
        target_view: &TextureView,
    ) {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: target_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: self.depth_texture.view(),
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Models
        for model in self.model_cache.values() {
            render_pass.set_pipeline(
                model
                    .materials()
                    .first()
                    .unwrap()
                    .pipeline()
                    .render_pipeline(),
            );

            render_pass.set_bind_group(0, model.materials().first().unwrap().bind_group(), &[]);
            render_pass.set_bind_group(1, world.active_camera().camera_bind_group(), &[]);
            render_pass.set_bind_group(2, world.light_store().point_light_bind_group(), &[]);
            render_pass.set_bind_group(
                3,
                self.world_environment.as_ref().unwrap().bind_group(),
                &[],
            );

            render_pass.set_vertex_buffer(0, model.mesh().vertex_buffer().slice(..));
            render_pass.set_vertex_buffer(1, model.instance_buffer().slice(..));
            render_pass
                .set_index_buffer(model.mesh().index_buffer().slice(..), IndexFormat::Uint32);

            render_pass.draw_indexed(0..model.mesh().index_count(), 0, 0..model.instance_count());
        }
    }

    async fn process_change_list(
        &mut self,
        change_list: ChangeList,
        world: &World,
        device: &Device,
        queue: &Queue,
    ) {
        for change in change_list {
            match change {
                Change::Clear(change_type) => match change_type {
                    ChangeType::Model { label: _ } => self.model_cache.clear(),
                    ChangeType::Light { label: _ } => {
                        // TODO: Once properly caching this entity
                    }
                    ChangeType::Camera { label: _ } => {
                        // TODO: Once properly caching this entity
                    }
                    ChangeType::WorldEnvironment { label: _ } => {
                        // TODO: Once properly caching this entity
                    }
                },
                Change::Added(change_type) | Change::Changed(change_type) => match change_type {
                    ChangeType::Model { label } => {
                        let lbl = label.expect("Change is expected to have a label set!");

                        let descriptor = world
                            .model_store()
                            .get(&lbl)
                            .expect("Attempting to realize model that doesn't exist as descriptor!")
                            .read()
                            .await;

                        match Model::from_descriptor(
                            &descriptor,
                            &self.surface_format,
                            device,
                            queue,
                            &self.app_name,
                            Some(&self.cache_state),
                        ) {
                            Ok(model) => {
                                self.model_cache.insert(lbl, model);
                            }
                            Err(e) => error!("Failed realizing model from ChangeList: {:?}", e),
                        }
                    }
                    ChangeType::Light { label } => {
                        // TODO: Once properly caching this entity
                    }
                    ChangeType::Camera { label } => {
                        // TODO: Once properly caching this entity
                    }
                    ChangeType::WorldEnvironment { label } => {
                        // TODO: Once properly caching this entity
                    }
                },
                Change::Removed(change_type) => match change_type {
                    ChangeType::Model { label } => {
                        let lbl = label.expect("Change is expected to have a label set!");
                        self.model_cache.remove(&lbl);
                    }
                    ChangeType::Light { label } => {
                        // TODO: Once properly caching this entity
                    }
                    ChangeType::Camera { label } => {
                        // TODO: Once properly caching this entity
                    }
                    ChangeType::WorldEnvironment { label } => {
                        // TODO: Once properly caching this entity
                    }
                },
            }
        }
    }
}

#[async_trait]
impl Renderer for CachingDirectRenderer {
    fn new(
        surface_texture_format: wgpu::TextureFormat,
        resolution: cgmath::Vector2<u32>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        app_name: &str,
    ) -> Self {
        let depth_texture =
            Texture::from_descriptor(&TextureDescriptor::Depth(resolution), device, queue)
                .expect("Depth texture realization failed!");

        Self {
            app_name: app_name.to_string(),
            surface_format: surface_texture_format,
            depth_texture,
            model_cache: HashMap::new(),
            world_environment: None,
            world_environment_pipeline: None,
            cache_state: CacheState::new(),
        }
    }

    async fn change_surface_texture_format(
        &mut self,
        surface_texture_format: TextureFormat,
        _device: &Device,
        _queue: &Queue,
    ) {
        // Set the format internally
        self.surface_format = surface_texture_format;
    }

    async fn change_resolution(
        &mut self,
        resolution: Vector2<u32>,
        device: &Device,
        queue: &Queue,
    ) {
        // Remake the depth texture with the new size
        self.depth_texture = Texture::depth_texture(&resolution, device, queue);
    }

    async fn render(
        &mut self,
        target_view: &TextureView,
        device: &Device,
        queue: &Queue,
        world: &World, // TODO: Make generic if possible for other ECS like systems
    ) {
        if self.world_environment.is_none() || self.world_environment.is_none() {
            self.world_environment = Some(Material::from_descriptor(
                world.world_environment(),
                &self.surface_format,
                device,
                queue,
                &self.app_name,
                Some(&self.cache_state),
            ).expect("TODO! Should never happen and will be replaced once change list WorldChanges are implemented."));

            self.world_environment_pipeline = Some(Pipeline::from_descriptor(
                self.world_environment
                    .as_ref()
                    .unwrap()
                    .pipeline_descriptor(),
                &self.surface_format,
                device,
                queue,
                Some(&self.cache_state),
            ).expect("TODO! Should never happen and will be replaced once change list WorldChanges are implemented."));
        }

        self.process_change_list(world.take_change_list().await, world, device, queue)
            .await;

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        {
            self.render_skybox(world, &mut encoder, target_view);

            self.render_models(world, &mut encoder, target_view);
        }

        queue.submit(Some(encoder.finish()));
    }
}
