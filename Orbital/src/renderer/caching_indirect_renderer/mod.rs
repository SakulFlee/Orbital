use std::sync::Arc;

use async_trait::async_trait;
use cgmath::Vector2;
use hashbrown::HashMap;
use log::{debug, warn};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    include_wgsl, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBindingType, BufferUsages,
    CommandBuffer, CommandEncoderDescriptor, ComputePassDescriptor, ComputePipeline,
    ComputePipelineDescriptor, Device, IndexFormat, LoadOp, MaintainBase, Operations,
    PipelineLayoutDescriptor, Queue, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, ShaderModuleDescriptor, ShaderStages, StoreOp, TextureFormat,
    TextureView,
};

use crate::cache::Cache;
use crate::log::error;
use crate::resources::descriptors::{
    MaterialDescriptor, MeshDescriptor, ModelDescriptor, PipelineDescriptor, ShaderDescriptor,
};
use crate::resources::realizations::{Material, Mesh, Model, Shader};
use crate::resources::{
    descriptors::TextureDescriptor,
    realizations::{Pipeline, Texture},
};
use crate::variant::Variant;
use crate::world::{Change, ChangeList, ChangeType, Message, World};

use super::{DrawIndexedIndirect, Renderer};

pub struct CachingIndirectRenderer {
    app_name: String,
    surface_format: TextureFormat,
    depth_texture: Texture,
    world_environment: Option<Material>,
    world_environment_pipeline: Option<Pipeline>,
    model_cache: HashMap<String, Model>,
    mesh_cache: Cache<Arc<MeshDescriptor>, Mesh>,
    material_cache: Cache<Arc<MaterialDescriptor>, Material>,
    texture_cache: Cache<Arc<TextureDescriptor>, Texture>,
    pipeline_cache: Cache<Arc<PipelineDescriptor>, Pipeline>,
    shader_cache: Cache<Arc<ShaderDescriptor>, Shader>,
    debug_wireframes_enabled: bool,
    debug_bounding_box_wireframe_enabled: bool,
}

impl CachingIndirectRenderer {
    pub fn bind_group_layout_descriptor_frustum_culling_bounding_box_buffer(
    ) -> BindGroupLayoutDescriptor<'static> {
        BindGroupLayoutDescriptor {
            label: Some("Mip Buffer Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        }
    }

    fn make_compute_pipeline(
        bind_group_layouts: &[&BindGroupLayout],
        shader_module_descriptor: ShaderModuleDescriptor,
        shader_entrypoint: &str,
        device: &Device,
    ) -> ComputePipeline {
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts,
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(shader_module_descriptor);

        device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Frustum Culling Indirect Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some(shader_entrypoint),
            compilation_options: Default::default(),
            cache: None,
        })
    }

    fn frustum_culling_to_indirect_draw_buffers(
        &self,
        world: &World,
        device: &Device,
        queue: &Queue,
    ) -> HashMap<String, Buffer> {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Frustum Culling to Indirect Drawing Compute Encoder"),
        });

        let bind_group_layout = device.create_bind_group_layout(
            &Self::bind_group_layout_descriptor_frustum_culling_bounding_box_buffer(),
        );

        let mut indirect_draw_buffers = HashMap::new();
        for (model_label, model) in &self.model_cache {
            let entry = indirect_draw_buffers.entry(model_label.clone()).insert(
                device.create_buffer_init(&BufferInitDescriptor {
                    // Initial indirect draw data being "draw nothing at all"
                    contents: &DrawIndexedIndirect::new(
                        model.mesh().index_count(),
                        model.instance_count(),
                    )
                    .to_binary_data(),
                    usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::INDIRECT,
                    label: Some("Indirect Draw Buffer"),
                }),
            );
            let indirect_draw_buffer = entry.get();

            let bounding_box_buffer = match model.mesh().bounding_box_buffer() {
                Some(buffer) => buffer,
                None => {
                    // If the model doesn't have a bounding box buffer, we can't, and shouldn't, frustum cull it.
                    // The default indirect draw buffer is filled with the necessary data to always render the model.
                    // Thus, if the model doesn't have a bounding box buffer, we can skip the whole step.
                    continue;
                }
            };

            let pipeline = Self::make_compute_pipeline(
                &[&bind_group_layout],
                include_wgsl!("frustum_culling_to_indirect_drawing.wgsl"),
                "main",
                device,
            );

            let bind_group = device.create_bind_group(&BindGroupDescriptor {
                label: Some("Frustum Culling to Indirect Drawing"),
                layout: &bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::Buffer(
                            world.active_camera().buffer().as_entire_buffer_binding(),
                        ),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Buffer(
                            bounding_box_buffer.as_entire_buffer_binding(),
                        ),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::Buffer(
                            indirect_draw_buffer.as_entire_buffer_binding(),
                        ),
                    },
                ],
            });

            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Frustum Culling to Indirect Drawing"),
                ..Default::default()
            });

            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(1, 1, 1);

            drop(pass);
        }

        queue.submit(vec![encoder.finish()]);

        // device.poll(MaintainBase::Wait);

        indirect_draw_buffers
    }

    fn render_debug_bounding_boxes(
        &mut self,
        world: &World,
        device: &Device,
        queue: &Queue,
        target_view: &TextureView,
    ) -> CommandBuffer {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Debug Bounding Box Encoder"),
        });

        let pipeline = Pipeline::from_descriptor(
            &PipelineDescriptor::debug_bounding_box(),
            &self.surface_format,
            device,
            queue,
            Some(&mut self.shader_cache),
        )
        .expect("Setting up debug debug bounding box pipeline failed!");

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Debug Bounding Box"),
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

        for model in self.model_cache.values() {
            let bounding_box = match model.mesh().bounding_box() {
                Some(bounding_box) => bounding_box,
                None => continue, // Skip models without bounding boxes
            };

            render_pass.set_pipeline(pipeline.render_pipeline());

            render_pass.set_bind_group(0, world.active_camera().bind_group(), &[]);

            let (vertex_buffer, index_buffer) =
                bounding_box.to_debug_bounding_box_wireframe_buffers(device);

            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, model.instance_buffer().slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);

            render_pass.draw_indexed(0..bounding_box.to_indices().len() as u32, 0, 0..1);
        }

        drop(render_pass);
        encoder.finish()
    }

    fn render_debug_wireframes(
        &mut self,
        world: &World,
        device: &Device,
        queue: &Queue,
        target_view: &TextureView,
    ) -> CommandBuffer {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Debug Bounding Box Encoder"),
        });

        let pipeline = Pipeline::from_descriptor(
            &PipelineDescriptor::default_wireframe(),
            &self.surface_format,
            device,
            queue,
            Some(&mut self.shader_cache),
        )
        .expect("Setting up debug wireframe pipeline failed!");

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Debug Bounding Box"),
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

        for model in self.model_cache.values() {
            render_pass.set_pipeline(pipeline.render_pipeline());

            render_pass.set_bind_group(0, model.material().bind_group(), &[]);
            render_pass.set_bind_group(1, world.active_camera().bind_group(), &[]);
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

        drop(render_pass);
        encoder.finish()
    }

    fn render_skybox(
        &self,
        world: &World,
        device: &Device,
        target_view: &TextureView,
    ) -> CommandBuffer {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Skybox Encoder"),
        });

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Skybox RenderPass"),
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
        render_pass.set_bind_group(1, world.active_camera().bind_group(), &[]);
        render_pass.draw(0..3, 0..1);

        drop(render_pass);
        encoder.finish()
    }

    fn render_models(
        &self,
        world: &World,
        device: &Device,
        target_view: &TextureView,
        indirect_draw_buffers: HashMap<String, Buffer>,
    ) -> CommandBuffer {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Model Encoder"),
        });

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Model RenderPass"),
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
        for (model_label, model) in &self.model_cache {
            render_pass.set_pipeline(model.material().pipeline().render_pipeline());

            render_pass.set_bind_group(0, model.material().bind_group(), &[]);
            render_pass.set_bind_group(1, world.active_camera().bind_group(), &[]);
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

            let indirect_draw_buffer = indirect_draw_buffers.get(model_label).expect(&format!(
                "Indirect draw buffer is missing for model with label '{}'!",
                model_label
            ));
            render_pass.draw_indexed_indirect(indirect_draw_buffer, 0);
        }

        drop(render_pass);
        encoder.finish()
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
                            Some(&mut self.mesh_cache),
                            Some(&mut self.material_cache),
                            Some(&mut self.texture_cache),
                            Some(&mut self.pipeline_cache),
                            Some(&mut self.shader_cache),
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
impl Renderer for CachingIndirectRenderer {
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
            mesh_cache: Cache::new(),
            material_cache: Cache::new(),
            texture_cache: Cache::new(),
            pipeline_cache: Cache::new(),
            shader_cache: Cache::new(),
            debug_wireframes_enabled: false,
            debug_bounding_box_wireframe_enabled: false,
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

    // TODO: Sort the whole DEBUG statements better ...
    async fn on_message(&mut self, message: Message) {
        if let Some(Variant::Empty) = message.get("debug_wireframes_enabled") {
            self.debug_wireframes_enabled = !self.debug_wireframes_enabled;
            debug!(
                "Debug wireframes enabled: {}",
                self.debug_wireframes_enabled
            );
            return;
        }

        if let Some(Variant::Empty) = message.get("debug_bounding_box_wireframe_enabled") {
            self.debug_bounding_box_wireframe_enabled = !self.debug_bounding_box_wireframe_enabled;
            debug!(
                "Debug bounding box wireframes enabled: {}",
                self.debug_bounding_box_wireframe_enabled
            );
            return;
        }

        warn!(
            "Received message that didn't match any receiver: {:?}",
            message
        );
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
                Some(&mut self.texture_cache),
                Some(&mut self.pipeline_cache),
                Some(&mut self.shader_cache),
            ).expect("TODO! Should never happen and will be replaced once change list WorldChanges are implemented."));

            self.world_environment_pipeline = Some(Pipeline::from_descriptor(
                self.world_environment
                    .as_ref()
                    .unwrap()
                    .pipeline_descriptor(),
                &self.surface_format,
                device,
                queue,
                Some(&mut self.shader_cache),
            ).expect("TODO! Should never happen and will be replaced once change list WorldChanges are implemented."));
        }

        self.process_change_list(world.take_change_list().await, world, device, queue)
            .await;

        let indirect_draw_buffers =
            self.frustum_culling_to_indirect_draw_buffers(world, device, queue);

        let mut queue_submissions = Vec::new();

        // TODO: Using multiple encoders/passes seems to be a performance hit.
        queue_submissions.push(self.render_skybox(world, device, target_view));
        queue_submissions.push(self.render_models(
            world,
            device,
            target_view,
            indirect_draw_buffers,
        ));

        if self.debug_wireframes_enabled {
            queue_submissions.push(self.render_debug_wireframes(world, device, queue, target_view));
        }

        if self.debug_bounding_box_wireframe_enabled {
            queue_submissions.push(self.render_debug_bounding_boxes(
                world,
                device,
                queue,
                target_view,
            ));
        }

        queue.submit(queue_submissions);
    }
}
