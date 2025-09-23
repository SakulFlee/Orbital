use std::sync::OnceLock;
use std::time::{Duration, Instant};

use crate::element::{CameraEvent, ModelEvent, WorldEvent};
use crate::importer::Importer;
use crate::resources::{Camera, CameraDescriptor, IblBrdf, Model, Texture, WorldEnvironment};
use cgmath::Vector2;
use log::debug;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBindingType, Device, Queue,
    SamplerBindingType, ShaderStages, TextureFormat, TextureSampleType, TextureUsages,
    TextureViewDimension,
};

mod store;
pub use store::*;

pub struct World {
    model_store: ModelStore,
    camera_store: CameraStore,
    environment_store: EnvironmentStore,
    light_store: LightStore,
    last_cleanup: Instant,
    importer: Option<Importer>,
    ibl_brdf: Option<Texture>,
    /// The _Engine_ [`BindGroup`].
    /// > This may also be called _World_ [`BindGroup`]!
    ///
    /// Any relevant _Engine_ resources, such as the Camera and IBL, are contained here.
    world_bind_group: Option<BindGroup>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn make_world_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("World BindGroup Layout"),
            entries: &[
                // Camera
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::all(),
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // IBL Diffuse
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::all(),
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::Cube,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::all(),
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // IBL Specular
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::all(),
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::Cube,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::all(),
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // IBL BRDF
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStages::all(),
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 6,
                    visibility: ShaderStages::all(),
                    ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                    count: None,
                },
                // Light Store (Storage Buffer)
                BindGroupLayoutEntry {
                    binding: 13,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }

    pub fn new() -> Self {
        Self {
            model_store: ModelStore::new(),
            camera_store: CameraStore::new(),
            environment_store: EnvironmentStore::new(),
            light_store: LightStore::new(),
            last_cleanup: Instant::now(),
            importer: Some(Importer::new(4)),
            world_bind_group: None,
            ibl_brdf: None,
        }
    }

    pub fn model_store(&self) -> &ModelStore {
        &self.model_store
    }

    pub fn model_store_mut(&mut self) -> &mut ModelStore {
        &mut self.model_store
    }

    pub fn camera_store(&self) -> &CameraStore {
        &self.camera_store
    }

    pub fn camera_store_mut(&mut self) -> &mut CameraStore {
        &mut self.camera_store
    }

    pub fn environment_store(&self) -> &EnvironmentStore {
        &self.environment_store
    }

    pub fn environment_store_mut(&mut self) -> &mut EnvironmentStore {
        &mut self.environment_store
    }

    pub async fn update(&mut self, world_events: Vec<WorldEvent>) {
        // Process through other world events
        for world_event in world_events {
            self.process_event(world_event);
        }

        // Take temporary ownership of importer
        let mut importer = self.importer.take().unwrap();
        // Call async future early so it might be done by the time we check it
        let importer_results = importer.update().await;
        // Put importer back
        self.importer = Some(importer);

        for importer_result in importer_results {
            for model in importer_result.models {
                self.process_event(WorldEvent::Model(ModelEvent::Spawn(model)));
            }
            for camera in importer_result.cameras {
                self.process_event(WorldEvent::Camera(CameraEvent::Spawn(camera)));
            }
        }

        // Needs to be at most the same as the cache timeout time!
        // Otherwise, cache cleanup will never be efficient.
        if self.last_cleanup.elapsed() >= Duration::from_secs(5) {
            self.model_store.cleanup(); // TODO
            self.camera_store.cleanup();

            self.last_cleanup = Instant::now();
        }
    }

    fn recreate_bind_group(&mut self, device: &Device, queue: &Queue) {
        // Create light buffer first to avoid borrowing issues
        self.light_store.create_light_buffer(device, queue);

        // Get the light buffer binding first to avoid borrowing conflicts
        let light_buffer_binding = {
            let light_buffer = self.light_store.light_buffer();
            light_buffer
                .map(|buffer| buffer.as_entire_buffer_binding())
                .unwrap_or_else(|| {
                    // Create a dummy buffer binding if no lights
                    static FALLBACK_ONCE: OnceLock<Buffer> = OnceLock::new();
                    let fallback = FALLBACK_ONCE.get_or_init(|| {
                        device.create_buffer(&wgpu::BufferDescriptor {
                            label: Some("Fallback Light Buffer"),
                            size: 4,
                            usage: wgpu::BufferUsages::STORAGE,
                            mapped_at_creation: false,
                        })
                    });
                    fallback.as_entire_buffer_binding()
                })
        };

        if self.ibl_brdf.is_none() {
            self.ibl_brdf = Some(IblBrdf::generate(device, queue).texture());
        }
        let local_ibl_brdf = self.ibl_brdf.as_ref().unwrap();
        let (ibl_brdf_view, ibl_brdf_sampler) = (local_ibl_brdf.view(), local_ibl_brdf.sampler());

        let (
            world_environment_ibl_diffuse_view,
            world_environment_ibl_diffuse_sampler,
            world_environment_ibl_specular_view,
            world_environment_ibl_specular_sampler,
        ) = match self.environment_store().world_environment() {
            Some(x) => (
                x.ibl_diffuse().view(),
                x.ibl_diffuse().sampler(),
                x.ibl_specular().view(),
                x.ibl_specular().sampler(),
            ),
            None => {
                debug!("Attempting to recreate World BindGroup without an active WorldEnvironment! Using a default fallback.");
                static FALLBACK_ONCE: OnceLock<(Texture, Texture)> = OnceLock::new();
                let (fallback_ibl_diffuse, fallback_ibl_specular) =
                    FALLBACK_ONCE.get_or_init(|| {
                        (
                            Texture::create_empty_cube_texture(
                                Some("default IBL diffuse"),
                                Vector2::new(1, 1),
                                TextureFormat::R8Unorm,
                                TextureUsages::TEXTURE_BINDING,
                                1,
                                device,
                            ),
                            Texture::create_empty_cube_texture(
                                Some("default IBL specular"),
                                Vector2::new(1, 1),
                                TextureFormat::R8Unorm,
                                TextureUsages::TEXTURE_BINDING,
                                1,
                                device,
                            ),
                        )
                    });

                (
                    fallback_ibl_diffuse.view(),
                    fallback_ibl_diffuse.sampler(),
                    fallback_ibl_specular.view(),
                    fallback_ibl_specular.sampler(),
                )
            }
        };

        let active_camera_buffer = match self.camera_store().get_realized_active_camera() {
            Some(x) => x.camera_buffer().as_entire_buffer_binding(),
            None => {
                debug!("Attempting to recreate World BindGroup without an active Camera! Using a default fallback.");
                static FALLBACK_ONCE: OnceLock<Camera> = OnceLock::new();
                let fallback = FALLBACK_ONCE.get_or_init(|| {
                    Camera::from_descriptor(CameraDescriptor::default(), device, queue)
                });
                fallback.camera_buffer().as_entire_buffer_binding()
            }
        };

        let bind_group_layout = Self::make_world_bind_group_layout(device);
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("World Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(active_camera_buffer),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(world_environment_ibl_diffuse_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(world_environment_ibl_diffuse_sampler),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(world_environment_ibl_specular_view),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::Sampler(world_environment_ibl_specular_sampler),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: BindingResource::TextureView(ibl_brdf_view),
                },
                BindGroupEntry {
                    binding: 6,
                    resource: BindingResource::Sampler(ibl_brdf_sampler),
                },
                BindGroupEntry {
                    binding: 13,
                    resource: BindingResource::Buffer(light_buffer_binding),
                },
            ],
        });

        self.world_bind_group = Some(bind_group);
    }

    pub fn process_event(&mut self, event: WorldEvent) {
        match event {
            WorldEvent::Model(model_event) => self.model_store.handle_event(model_event),
            WorldEvent::Camera(camera_event) => self.camera_store.handle_event(camera_event),
            WorldEvent::Environment(environment_event) => {
                self.environment_store.handle_event(environment_event);
            }
            WorldEvent::Light(light_event) => {
                self.light_store.handle_event(light_event);
            }
            WorldEvent::Import(import_task) => {
                self.importer.as_mut().unwrap().register_task(import_task);
            }
            WorldEvent::Clear => {
                self.model_store.clear(); // TODO
                self.camera_store.clear();
                self.environment_store.clear();
                self.light_store.clear();
            }
        }
    }

    pub fn prepare_render(
        &mut self,
        surface_texture_format: &TextureFormat,
        device: &Device,
        queue: &Queue,
    ) {
        self.model_store.process_bounding_boxes(device);
        self.model_store
            .realize_and_cache(surface_texture_format, device, queue);
        self.camera_store.realize_and_cache(device, queue);
        if let Err(e) =
            self.environment_store
                .realize_and_cache(surface_texture_format, device, queue)
        {
            panic!("Failed to realize environment: {e}");
        }
        self.light_store.realize_and_cache(device, queue);

        self.recreate_bind_group(device, queue);
    }

    pub fn retrieve_render_resources(
        &self,
    ) -> (Option<&BindGroup>, Option<&WorldEnvironment>, Vec<&Model>) {
        // TODO: Use proper bounding box checking!
        let bounding_boxes = self.model_store.get_bounding_boxes();
        let ids = bounding_boxes.keys().copied().collect::<Vec<_>>();
        let models = self.model_store.get_realizations(ids);

        (
            self.world_bind_group.as_ref(),
            self.environment_store().world_environment(),
            models,
        )
    }
}
