use std::{
    hash::{Hash, Hasher},
    mem,
    sync::{Arc, OnceLock},
};

use cgmath::Vector3;
use hashbrown::HashMap;
use orbital_shaders::Shaders;
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
