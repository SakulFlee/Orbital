use std::num::NonZero;

use cgmath::Vector2;
use image::{GenericImageView, ImageReader};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    AddressMode, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferUsages, Color,
    ColorTargetState, ColorWrites, ComputePassDescriptor, ComputePipeline,
    ComputePipelineDescriptor, Device, Extent3d, FilterMode, FragmentState, ImageCopyTexture,
    ImageDataLayout, LoadOp, MultisampleState, Operations, Origin3d, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, SamplerBindingType,
    SamplerDescriptor, ShaderStages, StorageTextureAccess, StoreOp, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages,
    TextureViewDescriptor, TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexFormat,
    VertexState, VertexStepMode,
};

use crate::{error::Error, resources::descriptors::WorldEnvironmentDescriptor};

use super::Texture;

mod processing_type;
pub use processing_type::*;

pub struct WorldEnvironment {
    skybox_cube_texture: Texture,
    diffuse_cube_texture: Option<Texture>,
}

impl WorldEnvironment {
    pub fn bind_group_layout_descriptor_equirectangular_to_cube_texture(
    ) -> BindGroupLayoutDescriptor<'static> {
        BindGroupLayoutDescriptor {
            label: Some("Equirectangular"),
            entries: &[
                // Input
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Output
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba32Float,
                        view_dimension: TextureViewDimension::D2Array,
                    },
                    count: None,
                },
            ],
        }
    }

    pub fn bind_group_layout_descriptor_processing() -> BindGroupLayoutDescriptor<'static> {
        BindGroupLayoutDescriptor {
            label: Some("WorldEnvironment Processing"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::Cube,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        }
    }

    pub fn from_descriptor(
        desc: &WorldEnvironmentDescriptor,
        device: &Device,
        queue: &Queue,
    ) -> Result<Self, Error> {
        let texture = match desc {
            WorldEnvironmentDescriptor::FromFile {
                cube_face_size,
                path,
            } => Self::radiance_hdr_file(path, *cube_face_size, device, queue),
            WorldEnvironmentDescriptor::FromData {
                cube_face_size,
                data,
                size,
            } => Ok(Self::radiance_hdr_vec(
                data,
                *size,
                *cube_face_size,
                device,
                queue,
            )),
        }?;

        let mut s = Self {
            skybox_cube_texture: texture,
            diffuse_cube_texture: None,
        };

        s.process_diffuse(device, queue);

        Ok(s)
    }

    fn create_empty_cube_texture(
        label: Option<&str>,
        size: Vector2<u32>,
        format: TextureFormat,
        usage: TextureUsages,
        device: &Device,
    ) -> Texture {
        let texture = device.create_texture(&TextureDescriptor {
            label,
            size: Extent3d {
                width: size.x,
                height: size.y,
                // A cube has 6 sides, so we need 6 layers
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor {
            label,
            dimension: Some(TextureViewDimension::Cube),
            array_layer_count: Some(6),
            ..Default::default()
        });

        let sampler = device.create_sampler(&SamplerDescriptor {
            label,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        Texture::from_existing(texture, view, sampler)
    }

    pub fn radiance_hdr_file(
        file_path: &str,
        dst_size: u32,
        device: &Device,
        queue: &Queue,
    ) -> Result<Texture, Error> {
        let img = ImageReader::open(file_path)
            .map_err(Error::IOError)?
            .decode()
            .map_err(Error::ImageError)?;

        let width = img.dimensions().0;
        let height = img.dimensions().1;

        let data = img
            .into_rgba32f()
            .iter()
            .map(|x| x.to_le_bytes())
            .collect::<Vec<_>>()
            .concat();

        Ok(Self::radiance_hdr_vec(
            &data,
            Vector2 {
                x: width,
                y: height,
            },
            dst_size,
            device,
            queue,
        ))
    }

    pub fn radiance_hdr_vec(
        data: &[u8],
        src_size: Vector2<u32>,
        dst_size: u32,
        device: &Device,
        queue: &Queue,
    ) -> Texture {
        let src_texture = Texture::make_texture(
            Some("Equirectangular SRC"),
            Vector2 {
                x: src_size.x,
                y: src_size.y,
            },
            TextureFormat::Rgba32Float,
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            FilterMode::Linear,
            AddressMode::ClampToEdge,
            device,
            queue,
        );

        queue.write_texture(
            ImageCopyTexture {
                texture: src_texture.texture(),
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(src_size.x * std::mem::size_of::<[f32; 4]>() as u32),
                rows_per_image: Some(src_size.y),
            },
            Extent3d {
                width: src_size.x,
                height: src_size.y,
                ..Default::default()
            },
        );

        let dst_texture = Self::create_empty_cube_texture(
            Some("Equirectangular DST"),
            Vector2 {
                x: dst_size,
                y: dst_size,
            },
            TextureFormat::Rgba32Float,
            TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            device,
        );

        // Needed only know for processing.
        // Actual rendering later uses the included View in
        // TextureViewDimension::Cube
        let dst_equirectangular_view = dst_texture.texture().create_view(&TextureViewDescriptor {
            label: Some("Equirectangular DST view"),
            dimension: Some(TextureViewDimension::D2Array),
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(
            &Self::bind_group_layout_descriptor_equirectangular_to_cube_texture(),
        );
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Equirectangular"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(src_texture.view()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&dst_equirectangular_view),
                },
            ],
        });

        let pipeline = Self::make_compute_pipeline(&bind_group_layout, device);

        let mut encoder = device.create_command_encoder(&Default::default());
        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("Equirectangular"),
            ..Default::default()
        });

        let workgroups = (dst_size + 15) / 16;
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(workgroups, workgroups, 6);

        drop(pass);
        queue.submit([encoder.finish()]);

        dst_texture
    }

    pub fn process_diffuse(&mut self, device: &Device, queue: &Queue) {
        let src_size = self.skybox_cube_texture.texture().size();
        let dst_size = Vector2::new(src_size.width, src_size.height);

        let dst = Self::create_empty_cube_texture(
            "Diffuse CubeMap".into(),
            dst_size,
            TextureFormat::Rgba32Float,
            TextureUsages::STORAGE_BINDING
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::RENDER_ATTACHMENT,
            device,
        );

        // Needed only know for processing.
        // Actual rendering later uses the included View in
        // TextureViewDimension::Cube
        let dst_view = dst.texture().create_view(&TextureViewDescriptor {
            label: Some("DST view for Rendering ONLY"),
            dimension: Some(TextureViewDimension::D2Array),
            ..Default::default()
        });

        let bind_group_layout =
            device.create_bind_group_layout(&Self::bind_group_layout_descriptor_processing());
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Diffuse CubeMap Processing"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(self.skybox_cube_texture.view()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(self.skybox_cube_texture.sampler()),
                },
            ],
        });

        let pipeline = Self::make_render_pipeline(
            &bind_group_layout,
            ProcessingType::Diffuse,
            dst.texture().format(),
            device,
        );

        let mut encoder = device.create_command_encoder(&Default::default());
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Diffuse Processing"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &dst_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: StoreOp::Store,
                },
            })],
            ..Default::default()
        });

        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.set_vertex_buffer(0, Self::make_processing_vertex_buffer(device).slice(..));
        pass.draw(0..2, 0..1);

        drop(pass);
        queue.submit([encoder.finish()]);

        self.diffuse_cube_texture = Some(dst);
    }

    fn make_compute_pipeline(
        bind_group_layout: &BindGroupLayout,
        device: &Device,
    ) -> ComputePipeline {
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(include_wgsl!("equirectangular.wgsl"));

        device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Equirectangular to CubeMap"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        })
    }

    fn make_processing_vertex_buffer(device: &Device) -> Buffer {
        device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Processing Vertex Buffer"),
            contents: &[
                // 0
                0f32.to_le_bytes(),
                0f32.to_le_bytes(),
                0f32.to_le_bytes(),
                // 1
                1f32.to_le_bytes(),
                0f32.to_le_bytes(),
                0f32.to_le_bytes(),
                // 2
                0f32.to_le_bytes(),
                0f32.to_le_bytes(),
                1f32.to_le_bytes(),
            ]
            .concat(),
            usage: BufferUsages::VERTEX,
        })
    }

    fn make_render_pipeline(
        bind_group_layout: &BindGroupLayout,
        processing_type: ProcessingType,
        format: TextureFormat,
        device: &Device,
    ) -> RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
        });

        let vertex_shader_module =
            device.create_shader_module(include_wgsl!("processing_cube_vertex.wgsl"));

        let fragment_shader_module = match processing_type {
            ProcessingType::Diffuse => {
                device.create_shader_module(include_wgsl!("processing_diffuse_fragment.wgsl"))
            }
            ProcessingType::Specular => todo!(),
        };

        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some(format!("{:?}", processing_type).as_str()),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &vertex_shader_module,
                entry_point: "entrypoint_vertex",
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[VertexBufferLayout {
                    array_stride: size_of::<[f32; 4]>() as u64,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[
                        // Position
                        VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: VertexFormat::Float32x3,
                        },
                    ],
                }],
            },
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module: &fragment_shader_module,
                entry_point: "entrypoint_fragment",
                targets: &[Some(ColorTargetState {
                    format,
                    blend: None,
                    write_mask: ColorWrites::COLOR,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            multiview: Some(NonZero::new(6).unwrap()),
            cache: None,
        })
    }

    pub fn skybox_cube_texture(&self) -> &Texture {
        &self.skybox_cube_texture
    }

    pub fn diffuse_cube_texture(&self) -> &Texture {
        self.diffuse_cube_texture.as_ref().unwrap()
    }
}
