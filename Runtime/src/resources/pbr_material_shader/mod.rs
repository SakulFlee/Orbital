use cgmath::{Vector3, Zero};
use wgpu::{TextureFormat, TextureSampleType, TextureUsages};

use crate::resources::{
    BufferDescriptor, MaterialShaderDescriptor, VariableType, {TextureDescriptor, TextureSize},
};

#[cfg(test)]
mod tests;

pub type PBRMaterial = PBRMaterialDescriptor;
pub type PBRMaterialDescriptor = PBRMaterialShaderDescriptor;

pub struct PBRMaterialShaderDescriptor {
    // --- General ---
    pub name: Option<String>,
    // --- PBR specific ---
    pub normal: TextureDescriptor,
    pub albedo: TextureDescriptor,
    pub albedo_factor: Vector3<f32>,
    pub metallic: TextureDescriptor,
    pub metallic_factor: f32,
    pub roughness: TextureDescriptor,
    pub roughness_factor: f32,
    pub occlusion: TextureDescriptor,
    pub emissive: TextureDescriptor,
    // --- Material specific ---
    /// This field serves as a configuration base for creating a `MaterialShaderDescriptor`.
    /// If set to `Some(...)`, its contents will be used as the base configuration.
    /// If set to `None`, a default implementation will be used instead.
    ///
    /// Important notes:
    /// Any explicitly changed field in this struct (excluding the name and variables) will be transferred to the descriptor.
    /// For the PBR material workflow to work correctly, the descriptor must set a specific set of variables.
    /// This is not changeable!
    ///
    /// If you need to customize the shader descriptor beyond this default configuration, consider implementing
    /// your own `Into<MaterialShaderDescriptor>` trait specialization and/or providing a custom shader implementation.
    ///
    /// 🚀 If you have any suggestions for improvement, feel free to open an issue!
    pub custom_material_shader: Option<MaterialShaderDescriptor>,
}

impl PBRMaterialShaderDescriptor {}

impl Default for PBRMaterialShaderDescriptor {
    fn default() -> Self {
        Self {
            name: Some("Default PBR Material Shader".into()),
            normal: TextureDescriptor::Data {
                pixels: vec![0, 0, 0, 0],
                size: TextureSize {
                    width: 1,
                    height: 1,
                    ..Default::default()
                },
                format: TextureFormat::Rgba8UnormSrgb,
                usages: TextureUsages::all(),
            },
            albedo: TextureDescriptor::Data {
                pixels: vec![0, 0, 0, 0],
                size: TextureSize {
                    width: 1,
                    height: 1,
                    ..Default::default()
                },
                format: TextureFormat::Rgba8UnormSrgb,
                usages: TextureUsages::all(),
            },
            albedo_factor: Vector3::zero(),
            metallic: TextureDescriptor::Data {
                pixels: vec![0],
                size: TextureSize {
                    width: 1,
                    height: 1,
                    ..Default::default()
                },
                format: TextureFormat::R8Unorm,
                usages: TextureUsages::all(),
            },
            metallic_factor: 0.0,
            roughness: TextureDescriptor::Data {
                pixels: vec![0],
                size: TextureSize {
                    width: 1,
                    height: 1,
                    ..Default::default()
                },
                format: TextureFormat::R8Unorm,
                usages: TextureUsages::all(),
            },
            roughness_factor: 0.0,
            occlusion: TextureDescriptor::Data {
                pixels: vec![0],
                size: TextureSize {
                    width: 1,
                    height: 1,
                    ..Default::default()
                },
                format: TextureFormat::R8Unorm,
                usages: TextureUsages::all(),
            },
            emissive: TextureDescriptor::Data {
                pixels: vec![0],
                size: TextureSize {
                    width: 1,
                    height: 1,
                    ..Default::default()
                },
                format: TextureFormat::R8Unorm,
                usages: TextureUsages::all(),
            },
            custom_material_shader: Default::default(),
        }
    }
}

impl From<PBRMaterialShaderDescriptor> for MaterialShaderDescriptor {
    fn from(val: PBRMaterialShaderDescriptor) -> Self {
        let mut base = val.custom_material_shader.unwrap_or_default();

        base.name = val.name;
        base.variables = vec![
            // Normal
            VariableType::Texture {
                descriptor: val.normal,
                sample_type: TextureSampleType::Float { filterable: true },
            },
            // Albedo
            VariableType::Texture {
                descriptor: val.albedo,
                sample_type: TextureSampleType::Float { filterable: true },
            },
            // Metallic
            VariableType::Texture {
                descriptor: val.metallic,
                sample_type: TextureSampleType::Float { filterable: true },
            },
            // Roughness
            VariableType::Texture {
                descriptor: val.roughness,
                sample_type: TextureSampleType::Float { filterable: true },
            },
            // Occlusion
            VariableType::Texture {
                descriptor: val.occlusion,
                sample_type: TextureSampleType::Float { filterable: true },
            },
            // Emissive
            VariableType::Texture {
                descriptor: val.emissive,
                sample_type: TextureSampleType::Float { filterable: true },
            },
            // Factors
            // Note: Combines all factors in one buffer
            VariableType::Buffer(BufferDescriptor {
                data: [
                    // Albedo Factor
                    val.albedo_factor.x.to_le_bytes(), // R
                    val.albedo_factor.y.to_le_bytes(), // G
                    val.albedo_factor.z.to_le_bytes(), // B
                    // Metallic Factor
                    val.metallic_factor.to_le_bytes(), // LUMA
                    // Roughness Factor
                    val.roughness_factor.to_le_bytes(), // LUMA
                    // Padding to reach 32
                    [0; 4],
                    [0; 4],
                    [0; 4],
                ]
                .as_flattened()
                .to_vec(),
                ..Default::default()
            }),
        ];

        base
    }
}
