use cgmath::Vector3;
use orbital_material_shader::{
    AlphaMode, MaterialShaderDescriptor, RenderQueue, VertexStageLayout,
};
use orbital_shader_core::VariableType;
use orbital_texture::{BufferDescriptor, FilterMode, TextureDescriptor, TextureSize};
use wgpu::{
    BlendState, Face, SamplerBindingType, TextureDimension, TextureFormat, TextureSampleType,
    TextureUsages, TextureViewDimension,
};

pub type PBRMaterial = PBRMaterialDescriptor;
pub type PBRMaterialDescriptor = PBRMaterialShaderDescriptor;

/// The root node names (from the math/engine/PBR libraries) that assemble the
/// full PBR material shader.
pub const PBR_NODES: &[&str] = &[
    // Structs
    "pbr_factors",
    "light_contribution",
    "pbr_data_struct",
    "pbr_bindings",
    "pbr_vertex",
    "pbr_surface",
    "shadow_slot",
    // Math
    "pi",
    "f0_default",
    "ambient_intensity",
    "aces_tone_map",
    "fresnel_schlick",
    "fresnel_schlick_roughness",
    "distribution_ggx",
    "schlick_smith_ggx",
    "slope_scaled_bias",
    // Engine structs / layouts
    "camera_uniform",
    "light_struct",
    "light_types",
    "shadow_types",
    "vertex_data_complex",
    "instance_data",
    "fragment_data",
    // PBR orchestration
    "spot_bias_scale",
    "hdr_tone_map_gamma_correction",
    "sample_normal_from_map",
    "pbr_data",
    "calculate_light_brdf",
    "calculate_ambient_ibl",
    "sample_shadow_2d_pcf",
    "compute_shadow_for_light",
    "calculate_light_contribution",
];

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
    pub alpha_mode: AlphaMode,
    pub alpha_factor: f32,
    // --- Material specific ---
    /// Configuration base for creating a `MaterialShaderDescriptor`.
    /// If `Some(...)`, its contents are used as the base configuration.
    /// If `None`, a default implementation is used instead.
    ///
    /// The PBR material workflow requires a specific set of variables, which is
    /// not changeable.
    pub custom_material_shader: Option<MaterialShaderDescriptor>,
}

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

                texture_dimension: TextureDimension::D2,
                texture_view_dimension: TextureViewDimension::D2,
                filter_mode: FilterMode::default(),
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
                texture_dimension: TextureDimension::D2,
                texture_view_dimension: TextureViewDimension::D2,
                filter_mode: FilterMode::default(),
            },
            albedo_factor: Vector3::new(1.0, 1.0, 1.0),
            metallic: TextureDescriptor::Data {
                pixels: vec![0],
                size: TextureSize {
                    width: 1,
                    height: 1,
                    ..Default::default()
                },
                format: TextureFormat::R8Unorm,
                usages: TextureUsages::all(),
                texture_dimension: TextureDimension::D2,
                texture_view_dimension: TextureViewDimension::D2,
                filter_mode: FilterMode::default(),
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
                texture_dimension: TextureDimension::D2,
                texture_view_dimension: TextureViewDimension::D2,
                filter_mode: FilterMode::default(),
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
                texture_dimension: TextureDimension::D2,
                texture_view_dimension: TextureViewDimension::D2,
                filter_mode: FilterMode::default(),
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
                texture_dimension: TextureDimension::D2,
                texture_view_dimension: TextureViewDimension::D2,
                filter_mode: FilterMode::default(),
            },
            alpha_mode: AlphaMode::Opaque,
            alpha_factor: 1.0,
            custom_material_shader: Default::default(),
        }
    }
}

impl From<PBRMaterialShaderDescriptor> for MaterialShaderDescriptor {
    fn from(val: PBRMaterialShaderDescriptor) -> Self {
        let mut base = match val.custom_material_shader {
            Some(base) => base,
            None => MaterialShaderDescriptor {
                nodes: PBR_NODES,
                raw_source: Some(include_str!("wgsl/pbr_entrypoints.wgsl").into()),
                vertex_stage_layouts: Some(vec![
                    VertexStageLayout::ComplexVertexData,
                    VertexStageLayout::InstanceData,
                ]),
                cull_mode: Some(Face::Front),
                ..MaterialShaderDescriptor::default()
            },
        };

        base.name = val.name;
        base.alpha_mode = val.alpha_mode;
        base.render_queue = if val.alpha_mode == AlphaMode::Blend {
            RenderQueue::Transparent
        } else {
            RenderQueue::Opaque
        };
        base.blend = if val.alpha_mode == AlphaMode::Blend {
            BlendState::ALPHA_BLENDING
        } else {
            BlendState::REPLACE
        };
        base.depth_write = val.alpha_mode != AlphaMode::Blend;
        base.cast_shadows = val.alpha_mode != AlphaMode::Blend;
        let alpha_cutoff = val.alpha_mode.cutoff().unwrap_or(0.0);
        base.variables = vec![
            // Normal
            VariableType::Texture {
                descriptor: Box::new(val.normal),
                sample_type: TextureSampleType::Float { filterable: true },
                sampler_binding_type: SamplerBindingType::Filtering,
            },
            // Albedo
            VariableType::Texture {
                descriptor: Box::new(val.albedo),
                sample_type: TextureSampleType::Float { filterable: true },
                sampler_binding_type: SamplerBindingType::Filtering,
            },
            // Metallic
            VariableType::Texture {
                descriptor: Box::new(val.metallic),
                sample_type: TextureSampleType::Float { filterable: true },
                sampler_binding_type: SamplerBindingType::Filtering,
            },
            // Roughness
            VariableType::Texture {
                descriptor: Box::new(val.roughness),
                sample_type: TextureSampleType::Float { filterable: true },
                sampler_binding_type: SamplerBindingType::Filtering,
            },
            // Occlusion
            VariableType::Texture {
                descriptor: Box::new(val.occlusion),
                sample_type: TextureSampleType::Float { filterable: true },
                sampler_binding_type: SamplerBindingType::Filtering,
            },
            // Emissive
            VariableType::Texture {
                descriptor: Box::new(val.emissive),
                sample_type: TextureSampleType::Float { filterable: true },
                sampler_binding_type: SamplerBindingType::Filtering,
            },
            // Factors
            VariableType::Buffer(BufferDescriptor {
                data: [
                    val.albedo_factor.x.to_le_bytes(),
                    val.albedo_factor.y.to_le_bytes(),
                    val.albedo_factor.z.to_le_bytes(),
                    val.metallic_factor.to_le_bytes(),
                    val.roughness_factor.to_le_bytes(),
                    val.alpha_factor.to_le_bytes(),
                    alpha_cutoff.to_le_bytes(),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn factors(descriptor: PBRMaterialShaderDescriptor) -> MaterialShaderDescriptor {
        descriptor.into()
    }

    fn factor_bytes(descriptor: &MaterialShaderDescriptor) -> &[u8] {
        match descriptor.variables.last().unwrap() {
            VariableType::Buffer(buffer) => &buffer.data,
            other => panic!("expected PBR factor buffer, got {other:?}"),
        }
    }

    #[test]
    fn pbr_factor_buffer_contains_alpha_and_cutoff() {
        let descriptor = factors(PBRMaterialShaderDescriptor::default());
        let bytes = factor_bytes(&descriptor);

        assert_eq!(bytes.len(), 32);
        assert_eq!(f32::from_le_bytes(bytes[20..24].try_into().unwrap()), 1.0);
        assert_eq!(f32::from_le_bytes(bytes[24..28].try_into().unwrap()), 0.0);
    }

    #[test]
    fn blend_mode_selects_transparent_queue() {
        let descriptor = factors(PBRMaterialShaderDescriptor {
            alpha_mode: AlphaMode::Blend,
            ..PBRMaterialShaderDescriptor::default()
        });

        assert_eq!(descriptor.render_queue, RenderQueue::Transparent);
        assert_eq!(descriptor.blend, BlendState::ALPHA_BLENDING);
        assert!(!descriptor.depth_write);
        assert!(!descriptor.cast_shadows);
    }

    #[test]
    fn mask_mode_keeps_opaque_queue_and_cutoff() {
        let descriptor = factors(PBRMaterialShaderDescriptor {
            alpha_mode: AlphaMode::mask(0.25),
            ..PBRMaterialShaderDescriptor::default()
        });
        let bytes = factor_bytes(&descriptor);

        assert_eq!(descriptor.render_queue, RenderQueue::Opaque);
        assert!(descriptor.depth_write);
        assert_eq!(f32::from_le_bytes(bytes[24..28].try_into().unwrap()), 0.25);
    }
}
