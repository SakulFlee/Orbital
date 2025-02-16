use std::{
    hash::{Hash, Hasher},
    mem,
    sync::{Arc, OnceLock},
};

use cgmath::Vector3;
use hashbrown::HashMap;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingResource, BindingType, BlendState, Buffer, BufferBindingType, Color, ColorTargetState,
    ColorWrites, CompareFunction, DepthStencilState, Device, Face, FragmentState, FrontFace,
    MultisampleState, PipelineLayout, PipelineLayoutDescriptor, PolygonMode, PrimitiveState,
    PrimitiveTopology, Queue, RenderPipelineDescriptor, SamplerBindingType, ShaderModule,
    ShaderModuleDescriptor, ShaderStages, StencilState, TextureFormat, TextureSampleType,
    VertexBufferLayout, VertexState,
};

use crate::{
    error::Error,
    resources::realizations::{Instance, Texture, Vertex},
};

use super::{
    shader, BufferDescriptor, SamplingType, TextureDescriptor, WorldEnvironmentDescriptor,
};

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum VariableType {
    Buffer(BufferDescriptor),
    Texture {
        descriptor: TextureDescriptor,
        sampler_type: TextureSampleType,
    },
    // TODO: BindingType::StorageTexture
}

#[derive(Debug)]
pub enum Variable {
    Buffer(Buffer),
    Texture(Texture),
}

pub type Variables = HashMap<u32, Variable>;

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum VertexStageLayout {
    SimpleVertexData,
    ComplexVertexData,
    InstanceData,
    Custom(VertexBufferLayout<'static>),
}

impl VertexStageLayout {
    pub fn vertex_buffer_layout(self) -> VertexBufferLayout<'static> {
        match self {
            VertexStageLayout::SimpleVertexData => Vertex::simple_vertex_buffer_layout_descriptor(),
            VertexStageLayout::ComplexVertexData => {
                Vertex::complex_vertex_buffer_layout_descriptor()
            }
            VertexStageLayout::InstanceData => Instance::vertex_buffer_layout_descriptor(),
            VertexStageLayout::Custom(vertex_buffer_layout) => vertex_buffer_layout,
        }
    }
}

pub trait ShaderDescriptor {
    fn shader_path(&self) -> &'static str;

    fn variables(&self) -> &Vec<VariableType>;

    fn stages(&self) -> ShaderStages;

    fn shader_module(&self, device: &Device) -> ShaderModule {
        // TODO: ShaderLib parsing here
        device.create_shader_module(ShaderModuleDescriptor {
            label: Some(&self.shader_path()),
            source: todo!(),
        })
    }

    fn bind_group_layout(
        &self,
        device: &Device,
        queue: &Queue,
    ) -> Result<(BindGroupLayout, Variables), Error> {
        let mut entries = Vec::new();
        let mut variables: Variables = Variables::new();

        let mut binding_count = 0;
        for var in self.variables() {
            match var {
                VariableType::Buffer(buffer_descriptor) => {
                    let buffer = device.create_buffer_init(&BufferInitDescriptor {
                        label: None,
                        usage: buffer_descriptor.usage,
                        contents: &buffer_descriptor.data,
                    });
                    variables.insert(binding_count, Variable::Buffer(buffer));

                    let entry = BindGroupLayoutEntry {
                        binding: binding_count,
                        visibility: self.stages(),
                        ty: BindingType::Buffer {
                            ty: buffer_descriptor.ty,
                            has_dynamic_offset: buffer_descriptor.has_dynamic_offset,
                            min_binding_size: buffer_descriptor.min_binding_size,
                        },
                        count: buffer_descriptor.count,
                    };
                    entries.push(entry);

                    binding_count += 1;
                }
                VariableType::Texture {
                    descriptor,
                    sampler_type: sample_type,
                } => {
                    let variable = variables.entry(binding_count).insert(Variable::Texture(
                        Texture::from_descriptor(descriptor, device, queue)?,
                    ));
                    let texture = if let Variable::Texture(texture) = variable.get() {
                        texture
                    } else {
                        unreachable!()
                    };
                    // Note: Skipping 2nd texture binding here as we only need to store the actual texture once!

                    let texture_binding = BindGroupLayoutEntry {
                        binding: binding_count,
                        visibility: self.stages(),
                        ty: BindingType::Texture {
                            sample_type: *sample_type,
                            view_dimension: *texture.view_dimension(),
                            multisampled: false,
                        },
                        count: None,
                    };
                    entries.push(texture_binding);
                    binding_count += 1;

                    let sampler_binding = BindGroupLayoutEntry {
                        binding: binding_count,
                        visibility: self.stages(),
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    };
                    entries.push(sampler_binding);
                    binding_count += 1;
                }
            }
        }

        Ok((
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some(&self.shader_path()),
                entries: &entries,
            }),
            variables,
        ))
    }

    fn bind_group(
        &self,
        device: &Device,
        queue: &Queue,
    ) -> Result<(BindGroup, BindGroupLayout, Variables), Error> {
        let (layout, variables) = self.bind_group_layout(device, queue)?;

        let mut binds = Vec::new();
        let mut binding_index = 0u32;

        for var in self.variables() {
            match var {
                VariableType::Buffer(_) => {
                    let buffer = if let Variable::Buffer(buffer) = variables
                        .get(&binding_index)
                        .expect("Expected Variable to exist!")
                    {
                        buffer
                    } else {
                        panic!("Expected Buffer but got unexpected type!");
                    };

                    binds.push(BindGroupEntry {
                        binding: binding_index,
                        resource: BindingResource::Buffer(buffer.as_entire_buffer_binding()),
                    });
                    binding_index += 1;
                }
                VariableType::Texture {
                    descriptor: _,
                    sampler_type: _,
                } => {
                    // Retrieve current texture and increment index.
                    // Why is this separate from binding_index?
                    // E.g. textures are always two bindings.
                    // Buffers are just one binding though.
                    // Thus, we need to keep track of the texture index individually.
                    let texture = if let Variable::Texture(texture) = variables
                        .get(&binding_index)
                        .expect("Expected Variable to exist!")
                    {
                        texture
                    } else {
                        panic!("Expected Texture but got unexpected type!");
                    };

                    binds.push(BindGroupEntry {
                        binding: binding_index,
                        resource: BindingResource::TextureView(texture.view()),
                    });
                    binding_index += 1;

                    binds.push(BindGroupEntry {
                        binding: binding_index,
                        resource: BindingResource::Sampler(texture.sampler()),
                    });
                    binding_index += 1;
                }
            }
        }

        Ok((
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&self.shader_path()),
                layout: &layout,
                entries: &binds,
            }),
            layout,
            variables,
        ))
    }
}

// TODO: More work needs to be done here, Materials first though!
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct ComputeShaderDescriptor {
    shader_path: &'static str,
    variables: Vec<VariableType>,
}

impl ShaderDescriptor for ComputeShaderDescriptor {
    fn shader_path(&self) -> &'static str {
        &self.shader_path
    }

    fn variables(&self) -> &Vec<VariableType> {
        &self.variables
    }

    fn stages(&self) -> ShaderStages {
        ShaderStages::COMPUTE
    }
}

pub struct EngineBindGroupLayout;
impl EngineBindGroupLayout {
    pub fn make_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Engine"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::all(),
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct MaterialShaderDescriptor {
    shader_path: &'static str,
    variables: Vec<VariableType>,
    entrypoint_vertex: &'static str,
    entrypoint_fragment: &'static str,
    vertex_stage_layouts: Vec<VertexStageLayout>,
    primitive_topology: PrimitiveTopology,
    front_face_order: FrontFace,
    cull_mode: Option<Face>,
    polygon_mode: PolygonMode,
    depth_stencil: bool,
}

impl MaterialShaderDescriptor {
    fn create_render_pipeline(
        &self,
        surface_format: &TextureFormat,
        device: &Device,
        queue: &Queue,
    ) -> Result<wgpu::RenderPipeline, Error> {
        let shader_module = self.shader_module(device);
        // TODO: Cache

        // Create pipeline layout and bind group
        let (layout, variables) = self.bind_group_layout(device, queue)?;

        let engine_bind_group_layout_once = OnceLock::new();
        let engine_bind_group_layout = engine_bind_group_layout_once
            .get_or_init(|| EngineBindGroupLayout::make_bind_group_layout(device));

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(&self.shader_path),
            bind_group_layouts: &[&engine_bind_group_layout, &layout],
            push_constant_ranges: &[],
        });

        let vertex_buffer_layouts = self
            .vertex_stage_layouts
            .clone()
            .into_iter()
            .map(|x| x.vertex_buffer_layout())
            .collect::<Vec<_>>();

        let depth_stencil = if self.depth_stencil {
            Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            })
        } else {
            None
        };

        let targets = [Some(ColorTargetState {
            format: *surface_format,
            blend: Some(BlendState::REPLACE),
            write_mask: ColorWrites::ALL,
        })];

        // Create the actual render pipeline
        let pipeline_desc = RenderPipelineDescriptor {
            label: Some(self.shader_path()),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader_module,
                entry_point: Some(self.entrypoint_vertex),
                buffers: &vertex_buffer_layouts,
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: "main".into(),
                targets: &targets,
                compilation_options: Default::default(),
            }),
            depth_stencil,
            primitive: PrimitiveState {
                topology: self.primitive_topology,
                strip_index_format: None,
                front_face: self.front_face_order,
                cull_mode: self.cull_mode,
                unclipped_depth: false,
                polygon_mode: self.polygon_mode,
                conservative: false,
            },
            cache: None,
            multiview: None,
            multisample: Default::default(),
        };

        Ok(device.create_render_pipeline(&pipeline_desc))
    }
}

impl ShaderDescriptor for MaterialShaderDescriptor {
    fn shader_path(&self) -> &'static str {
        &self.shader_path
    }

    fn variables(&self) -> &Vec<VariableType> {
        &self.variables
    }

    fn stages(&self) -> ShaderStages {
        ShaderStages::VERTEX_FRAGMENT
    }
}

impl Default for MaterialShaderDescriptor {
    fn default() -> Self {
        Self {
            shader_path: "shaders/default.wgsl", // TODO: Write!
            variables: Vec::new(),
            entrypoint_vertex: "entrypoint_vertex",
            entrypoint_fragment: "entrypoint_fragment",
            vertex_stage_layouts: vec![
                VertexStageLayout::SimpleVertexData,
                VertexStageLayout::InstanceData,
            ],
            primitive_topology: PrimitiveTopology::TriangleList,
            front_face_order: FrontFace::Ccw,
            cull_mode: Some(Face::Front),
            polygon_mode: PolygonMode::Fill,
            depth_stencil: true,
        }
    }
}

pub struct PBRMaterialDescriptor {
    pub normal: TextureDescriptor,
    pub albedo: TextureDescriptor,
    pub albedo_factor: Vector3<f32>,
    pub metallic: TextureDescriptor,
    pub metallic_factor: f32,
    pub roughness: TextureDescriptor,
    pub roughness_factor: f32,
    pub occlusion: TextureDescriptor,
    pub emissive: TextureDescriptor,
    pub custom_shader: Option<&'static str>,
}

impl Into<MaterialShaderDescriptor> for PBRMaterialDescriptor {
    fn into(self) -> MaterialShaderDescriptor {
        MaterialShaderDescriptor {
            shader_path: self.custom_shader.unwrap_or("shader/pbr.wgsl"), // TODO: Write/Make, possibly after ShaderLib (see above)
            variables: vec![
                // TODO: Verify SamplerTypes here
                VariableType::Texture {
                    descriptor: self.normal,
                    sampler_type: TextureSampleType::Uint,
                },
                VariableType::Texture {
                    descriptor: self.albedo,
                    sampler_type: TextureSampleType::Uint,
                },
                VariableType::Texture {
                    descriptor: self.metallic,
                    sampler_type: TextureSampleType::Uint,
                },
                VariableType::Texture {
                    descriptor: self.roughness,
                    sampler_type: TextureSampleType::Uint,
                },
                VariableType::Texture {
                    descriptor: self.occlusion,
                    sampler_type: TextureSampleType::Uint,
                },
                VariableType::Texture {
                    descriptor: self.emissive,
                    sampler_type: TextureSampleType::Uint,
                },
                VariableType::Buffer(BufferDescriptor {
                    data: [
                        // Albedo Factor
                        self.albedo_factor.x.to_le_bytes(), // R
                        self.albedo_factor.y.to_le_bytes(), // G
                        self.albedo_factor.z.to_le_bytes(), // B
                        // Metallic Factor
                        self.metallic_factor.to_le_bytes(), // LUMA
                        // Roughness Factor
                        self.roughness_factor.to_le_bytes(), // LUMA
                        // Padding to reach 32
                        [0; 4],
                        [0; 4],
                        [0; 4],
                    ]
                    .as_flattened()
                    .to_vec(),
                    ..Default::default()
                }),
            ],
            ..Default::default()
        }
    }
}

// // --- OLD ---

// #[derive(Debug, Clone, PartialEq)]
// pub enum MaterialDescriptor {
//     /// Creates a standard PBR (= Physically-Based-Rendering) material.
//     PBR {
//         normal: Arc<TextureDescriptor>,
//         albedo: Arc<TextureDescriptor>,
//         albedo_factor: Vector3<f32>,
//         metallic: Arc<TextureDescriptor>,
//         metallic_factor: f32,
//         roughness: Arc<TextureDescriptor>,
//         roughness_factor: f32,
//         occlusion: Arc<TextureDescriptor>,
//         emissive: Arc<TextureDescriptor>,
//         custom_shader: Option<ShaderMaterialDescriptor>,
//     },
//     // TODO
//     WorldEnvironment(WorldEnvironmentDescriptor),
//     Wireframe(Color),
// }

// impl MaterialDescriptor {
//     pub fn default_world_environment() -> MaterialDescriptor {
//         MaterialDescriptor::WorldEnvironment(WorldEnvironmentDescriptor::FromFile {
//             skybox_type: super::SkyboxType::Specular { lod: 0 },
//             cube_face_size: super::WorldEnvironmentDescriptor::DEFAULT_SIZE,
//             path: "Assets/HDRs/kloppenheim_02_puresky_4k.hdr",
//             sampling_type: SamplingType::ImportanceSampling,
//         })
//     }
// }

// impl Hash for MaterialDescriptor {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         mem::discriminant(self).hash(state);
//     }
// }

// impl Eq for MaterialDescriptor {}
