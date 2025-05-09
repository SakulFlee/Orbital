use buffer::BufferDescriptor;
use cgmath::{Vector3, Zero};
use material_shader::MaterialShaderDescriptor;
use shader::VariableType;
use texture::{TextureDescriptor, TextureSize};
use wgpu::{TextureFormat, TextureSampleType, TextureUsages};

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

impl Into<MaterialShaderDescriptor> for PBRMaterialShaderDescriptor {
    fn into(self) -> MaterialShaderDescriptor {
        let mut base = self.custom_material_shader.unwrap_or_default();

        base.name = self.name;
        base.variables = vec![
            // Normal
            VariableType::Texture {
                descriptor: self.normal,
                sampler_type: TextureSampleType::Float { filterable: true },
            },
            // Albedo
            VariableType::Texture {
                descriptor: self.albedo,
                sampler_type: TextureSampleType::Float { filterable: true },
            },
            // Metallic
            VariableType::Texture {
                descriptor: self.metallic,
                sampler_type: TextureSampleType::Float { filterable: true },
            },
            // Roughness
            VariableType::Texture {
                descriptor: self.roughness,
                sampler_type: TextureSampleType::Float { filterable: true },
            },
            // Occlusion
            VariableType::Texture {
                descriptor: self.occlusion,
                sampler_type: TextureSampleType::Float { filterable: true },
            },
            // Emissive
            VariableType::Texture {
                descriptor: self.emissive,
                sampler_type: TextureSampleType::Float { filterable: true },
            },
            // Factors
            // Note: Combines all factors in one buffer
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
        ];

        base
    }
}
