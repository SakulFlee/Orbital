use cgmath::Vector2;
use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BufferBindingType, CommandBuffer, CommandEncoderDescriptor, ComputePipeline,
    ComputePipelineDescriptor, Device, LoadOp, Operations, PipelineLayoutDescriptor, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, ShaderModuleDescriptor, ShaderStages, StoreOp,
    TextureFormat, TextureView,
};

use crate::resources::{Texture, WorldEnvironment};

mod skybox;
mod model;
mod frustum_check;

pub struct Renderer {
    surface_format: TextureFormat,
    depth_texture: Texture,
    world_environment: Option<WorldEnvironment>,
}

impl Renderer {
    fn new(
        surface_texture_format: TextureFormat,
        resolution: Vector2<u32>,
        device: &Device,
        queue: &Queue,
    ) -> Self {
        let depth_texture = Texture::depth_texture(&resolution, device, queue);

        Self {
            surface_format: surface_texture_format,
            depth_texture,
            world_environment: None,
        }
    }

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
                        ty: BufferBindingType::Storage { read_only: true },
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

    // fn frustum_culling(&self, world: &World, device: &Device, queue: &Queue) -> Option<Buffer> {
    //     if self.model_cache.is_empty() {
    //         // No models, means we don't have bounding boxes.
    //         // This also means we can't render, so just skip here ...

    //         return None;
    //     }

    //     let bounding_box_data = self
    //         .model_cache
    //         .values()
    //         .map(|x| x.mesh().bounding_box().to_binary_data())
    //         .collect::<Vec<_>>()
    //         .concat();
    //     let bounding_box_buffer = device.create_buffer_init(&BufferInitDescriptor {
    //         label: Some("Bounding Box Buffer"),
    //         contents: &bounding_box_data,
    //         usage: BufferUsages::STORAGE,
    //     });

    //     let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
    //         label: Some("Frustum Culling to Indirect Drawing Compute Encoder"),
    //     });

    //     let bind_group_layout = device.create_bind_group_layout(
    //         &Self::bind_group_layout_descriptor_frustum_culling_bounding_box_buffer(),
    //     );

    //     let mut indirect_draw_buffers = HashMap::new();
    //     for (model_label, model) in &self.model_cache {
    //         let entry = indirect_draw_buffers.entry(model_label.clone()).insert(
    //             device.create_buffer_init(&BufferInitDescriptor {
    //                 // Initial indirect draw data being "draw nothing at all"
    //                 contents: &IndirectIndexedDraw::new(
    //                     model.mesh().index_count(),
    //                     model.instance_count(),
    //                 )
    //                 .to_binary_data(),
    //                 usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::INDIRECT,
    //                 label: Some("Indirect Draw Buffer"),
    //             }),
    //         );
    //         let indirect_draw_buffer = entry.get();

    //         let bounding_box_buffer = match model.mesh().bounding_box_buffer() {
    //             Some(buffer) => buffer,
    //             None => {
    //                 // If the model doesn't have a bounding box buffer, we can't, and shouldn't, frustum cull it.
    //                 // The default indirect draw buffer is filled with the necessary data to always render the model.
    //                 // Thus, if the model doesn't have a bounding box buffer, we can skip the whole step.
    //                 continue;
    //             }
    //         };

    //         let pipeline = Self::make_compute_pipeline(
    //             &[&bind_group_layout],
    //             include_wgsl!("frustum_culling.wgsl"),
    //             "main",
    //             device,
    //         );

    //         let bind_group = device.create_bind_group(&BindGroupDescriptor {
    //             label: Some("Frustum Culling to Indirect Drawing"),
    //             layout: &bind_group_layout,
    //             entries: &[
    //                 BindGroupEntry {
    //                     binding: 0,
    //                     resource: BindingResource::Buffer(
    //                         world
    //                             .active_camera()
    //                             .camera_buffer()
    //                             .as_entire_buffer_binding(),
    //                     ),
    //                 },
    //                 BindGroupEntry {
    //                     binding: 1,
    //                     resource: BindingResource::Buffer(
    //                         bounding_box_buffer.as_entire_buffer_binding(),
    //                     ),
    //                 },
    //                 BindGroupEntry {
    //                     binding: 2,
    //                     resource: BindingResource::Buffer(
    //                         indirect_draw_buffer.as_entire_buffer_binding(),
    //                     ),
    //                 },
    //             ],
    //         });

    //         let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
    //             label: Some("Frustum Culling to Indirect Drawing"),
    //             ..Default::default()
    //         });

    //         pass.set_pipeline(&pipeline);
    //         pass.set_bind_group(0, &bind_group, &[]);
    //         pass.dispatch_workgroups(1, 1, 1);

    //         drop(pass);
    //     }

    //     queue.submit(vec![encoder.finish()]);

    //     // device.poll(MaintainBase::Wait);

    //     indirect_draw_buffers
    // }

    // fn render_debug_bounding_boxes(
    //     &mut self,
    //     world: &World,
    //     device: &Device,
    //     queue: &Queue,
    //     target_view: &TextureView,
    // ) -> CommandBuffer {
    //     let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
    //         label: Some("Debug Bounding Box Encoder"),
    //     });

    //     let pipeline = Pipeline::from_descriptor(
    //         &PipelineDescriptor::default_wireframe(),
    //         &self.surface_format,
    //         device,
    //         queue,
    //         Some(&mut self.shader_cache),
    //     )
    //     .expect("Setting up debug debug bounding box pipeline failed!");

    //     let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
    //         label: Some("Debug Bounding Box"),
    //         color_attachments: &[Some(RenderPassColorAttachment {
    //             view: target_view,
    //             resolve_target: None,
    //             ops: Operations {
    //                 load: LoadOp::Load,
    //                 store: StoreOp::Store,
    //             },
    //         })],
    //         depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
    //             view: self.depth_texture.view(),
    //             depth_ops: Some(Operations {
    //                 load: LoadOp::Clear(1.0),
    //                 store: StoreOp::Store,
    //             }),
    //             stencil_ops: None,
    //         }),
    //         timestamp_writes: None,
    //         occlusion_query_set: None,
    //     });

    //     for model in self.model_cache.values() {
    //         let bounding_box = match model.mesh().bounding_box() {
    //             Some(bounding_box) => bounding_box,
    //             None => continue, // Skip models without bounding boxes
    //         };

    //         render_pass.set_pipeline(pipeline.render_pipeline());

    //         render_pass.set_bind_group(0, world.active_camera().camera_bind_group(), &[]);

    //         let (vertex_buffer, index_buffer) =
    //             bounding_box.to_debug_bounding_box_wireframe_buffers(device);

    //         render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
    //         render_pass.set_vertex_buffer(1, model.instance_buffer().slice(..));
    //         render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);

    //         render_pass.draw_indexed(0..bounding_box.to_indices().len() as u32, 0, 0..1);
    //     }

    //     drop(render_pass);
    //     encoder.finish()
    // }

    // fn render_debug_wireframes(
    //     &mut self,
    //     world: &World,
    //     device: &Device,
    //     queue: &Queue,
    //     target_view: &TextureView,
    // ) -> CommandBuffer {
    //     let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
    //         label: Some("Debug Bounding Box Encoder"),
    //     });

    //     let pipeline = Pipeline::from_descriptor(
    //         &PipelineDescriptor::default_wireframe(),
    //         &self.surface_format,
    //         device,
    //         queue,
    //         Some(&mut self.shader_cache),
    //     )
    //     .expect("Setting up debug wireframe pipeline failed!");

    //     let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
    //         label: Some("Debug Bounding Box"),
    //         color_attachments: &[Some(RenderPassColorAttachment {
    //             view: target_view,
    //             resolve_target: None,
    //             ops: Operations {
    //                 load: LoadOp::Load,
    //                 store: StoreOp::Store,
    //             },
    //         })],
    //         depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
    //             view: self.depth_texture.view(),
    //             depth_ops: Some(Operations {
    //                 load: LoadOp::Clear(1.0),
    //                 store: StoreOp::Store,
    //             }),
    //             stencil_ops: None,
    //         }),
    //         timestamp_writes: None,
    //         occlusion_query_set: None,
    //     });

    //     for model in self.model_cache.values() {
    //         render_pass.set_pipeline(pipeline.render_pipeline());

    //         render_pass.set_bind_group(0, model.material().bind_group(), &[]);
    //         render_pass.set_bind_group(1, world.active_camera().camera_bind_group(), &[]);
    //         render_pass.set_bind_group(2, world.light_store().point_light_bind_group(), &[]);
    //         render_pass.set_bind_group(
    //             3,
    //             self.world_environment.as_ref().unwrap().bind_group(),
    //             &[],
    //         );

    //         render_pass.set_vertex_buffer(0, model.mesh().vertex_buffer().slice(..));
    //         render_pass.set_vertex_buffer(1, model.instance_buffer().slice(..));
    //         render_pass
    //             .set_index_buffer(model.mesh().index_buffer().slice(..), IndexFormat::Uint32);

    //         render_pass.draw_indexed(0..model.mesh().index_count(), 0, 0..model.instance_count());
    //     }

    //     drop(render_pass);
    //     encoder.finish()
    // }

    fn render_skybox(
        &self,
        target_view: &TextureView,
        device: &Device,
        queue: &Queue,
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
        render_pass.set_bind_group(1, world.active_camera().camera_bind_group(), &[]);
        render_pass.draw(0..3, 0..1);

        drop(render_pass);
        encoder.finish()
    }

    fn render_models(
        &self,
        world: &World,
        device: &Device,
        target_view: &TextureView,
        indirect_indexed_draw_buffer: Buffer,
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

        // TODO: Switch over to offset rendering using single buffer
        // Models
        for (index, model) in self.model_cache.values().enumerate() {
            for material in model.materials() {
                render_pass.set_pipeline(material.pipeline().render_pipeline());

                render_pass.set_bind_group(0, material.bind_group(), &[]);
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

                render_pass.draw_indexed(
                    0..model.mesh().index_count(),
                    0,
                    0..model.instance_count(),
                );

                render_pass.draw_indexed_indirect(
                    &indirect_indexed_draw_buffer,
                    (index * IndirectIndexedDraw::byte_space_requirement()) as u64,
                );
            }
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
        if self.world_environment.is_none() || self.world_environment_pipeline.is_none() {
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

        let indirect_indexed_draw_buffer = device.create_buffer_init(&BufferInitDescriptor {
            contents: &self
                .model_cache
                .values()
                .map(|x| IndirectIndexedDraw::new(x.mesh().index_count(), x.instance_count()))
                .map(|x| x.to_binary_data())
                .collect::<Vec<_>>()
                .concat(),
            usage: BufferUsages::STORAGE | BufferUsages::INDIRECT,
            label: Some("Indirect Indexed Draw Buffer"),
        });

        // let indirect_draw_buffers = self.frustum_culling(world, device, queue);

        let mut queue_submissions = Vec::new();

        // TODO: Using multiple encoders/passes seems to be a performance hit.
        queue_submissions.push(self.render_skybox(world, device, target_view));
        queue_submissions.push(self.render_models(
            world,
            device,
            target_view,
            indirect_indexed_draw_buffer,
        ));

        // if self.debug_wireframes_enabled {
        //     queue_submissions.push(self.render_debug_wireframes(world, device, queue, target_view));
        // }

        // if self.debug_bounding_box_wireframe_enabled {
        //     queue_submissions.push(self.render_debug_bounding_boxes(
        //         world,
        //         device,
        //         queue,
        //         target_view,
        //     ));
        // }

        queue.submit(queue_submissions);
    }
}
