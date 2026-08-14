use std::sync::Arc;

use cgmath::Vector3;
use orbital_resources::{
    FilterMode, MaterialShaderDescriptor, PBRMaterialShaderDescriptor, ShaderSource,
    TextureDescriptor, TextureSize, VertexStageLayout,
};
use wgpu::{
    PolygonMode, PrimitiveTopology, TextureUsages, VertexAttribute, VertexBufferLayout,
    VertexFormat, VertexStepMode,
};

const DEFAULT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

fn mat_texture_usages() -> TextureUsages {
    TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SceneMaterial {
    Color {
        albedo: [f32; 4],
        metallic: f32,
        roughness: f32,
    },
    PbrFile {
        path: String,
    },
    #[serde(rename = "Grid")]
    GridWireframe,
}

impl SceneMaterial {
    pub fn into_material_shader(self) -> Arc<MaterialShaderDescriptor> {
        match self {
            SceneMaterial::Color {
                albedo,
                metallic,
                roughness,
            } => {
                let a = albedo;
                let rgba = [
                    (a[0].clamp(0.0, 1.0) * 255.0) as u8,
                    (a[1].clamp(0.0, 1.0) * 255.0) as u8,
                    (a[2].clamp(0.0, 1.0) * 255.0) as u8,
                    (a[3].clamp(0.0, 1.0) * 255.0) as u8,
                ];

                let m = (metallic.clamp(0.0, 1.0) * 255.0) as u8;
                let r = (roughness.clamp(0.0, 1.0) * 255.0) as u8;

                let tex_1x1 = || TextureSize {
                    width: 1,
                    height: 1,
                    ..Default::default()
                };

                let tex_1x1_rgba = TextureDescriptor::Data {
                    pixels: rgba.to_vec(),
                    size: tex_1x1(),
                    format: DEFAULT_FORMAT,
                    usages: mat_texture_usages(),
                    texture_dimension: wgpu::TextureDimension::D2,
                    texture_view_dimension: wgpu::TextureViewDimension::D2,
                    filter_mode: FilterMode::default(),
                };

                let tex_1x1_r = TextureDescriptor::Data {
                    pixels: vec![m],
                    size: tex_1x1(),
                    format: wgpu::TextureFormat::R8Unorm,
                    usages: mat_texture_usages(),
                    texture_dimension: wgpu::TextureDimension::D2,
                    texture_view_dimension: wgpu::TextureViewDimension::D2,
                    filter_mode: FilterMode::default(),
                };

                let tex_1x1_r2 = TextureDescriptor::Data {
                    pixels: vec![r],
                    size: tex_1x1(),
                    format: wgpu::TextureFormat::R8Unorm,
                    usages: mat_texture_usages(),
                    texture_dimension: wgpu::TextureDimension::D2,
                    texture_view_dimension: wgpu::TextureViewDimension::D2,
                    filter_mode: FilterMode::default(),
                };

                let normal_tex = TextureDescriptor::Data {
                    pixels: vec![128, 128, 255, 255],
                    size: tex_1x1(),
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usages: mat_texture_usages(),
                    texture_dimension: wgpu::TextureDimension::D2,
                    texture_view_dimension: wgpu::TextureViewDimension::D2,
                    filter_mode: FilterMode::default(),
                };

                let occlusion_tex = TextureDescriptor::Data {
                    pixels: vec![128],
                    size: tex_1x1(),
                    format: wgpu::TextureFormat::R8Unorm,
                    usages: mat_texture_usages(),
                    texture_dimension: wgpu::TextureDimension::D2,
                    texture_view_dimension: wgpu::TextureViewDimension::D2,
                    filter_mode: FilterMode::default(),
                };

                let emissive_tex = TextureDescriptor::Data {
                    pixels: vec![0, 0, 0, 0],
                    size: tex_1x1(),
                    format: DEFAULT_FORMAT,
                    usages: mat_texture_usages(),
                    texture_dimension: wgpu::TextureDimension::D2,
                    texture_view_dimension: wgpu::TextureViewDimension::D2,
                    filter_mode: FilterMode::default(),
                };

                log::info!(
                    "Creating material: albedo=({:.2},{:.2},{:.2}) metallic={:.2} roughness={:.2}",
                    a[0],
                    a[1],
                    a[2],
                    metallic,
                    roughness,
                );

                let pbr = PBRMaterialShaderDescriptor {
                    name: Some("SceneMaterial::Color".into()),
                    normal: normal_tex,
                    albedo: tex_1x1_rgba,
                    // Values are baked into 1x1 textures; factors act as
                    // neutral multipliers (matching glTF importer pattern).
                    albedo_factor: Vector3::new(1.0, 1.0, 1.0),
                    metallic: tex_1x1_r,
                    metallic_factor: 1.0,
                    roughness: tex_1x1_r2,
                    roughness_factor: 1.0,
                    occlusion: occlusion_tex,
                    emissive: emissive_tex,
                    custom_material_shader: None,
                };

                Arc::new(pbr.into())
            }
            SceneMaterial::PbrFile { .. } => {
                // Fallback to default material for now
                Arc::new(MaterialShaderDescriptor::default())
            }
            SceneMaterial::GridWireframe => {
                let mut base = MaterialShaderDescriptor::default();
                base.shader_source = ShaderSource::Path("Shaders/wireframe.wgsl");
                base.entrypoint_vertex = "entrypoint_vertex";
                base.entrypoint_fragment = "entrypoint_fragment";
                base.vertex_stage_layouts = Some(vec![
                    VertexStageLayout::Custom(VertexBufferLayout {
                        array_stride: 56,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &[VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: VertexFormat::Float32x3,
                        }],
                    }),
                    VertexStageLayout::InstanceData,
                ]);
                base.primitive_topology = PrimitiveTopology::LineList;
                base.polygon_mode = PolygonMode::Fill;
                base.cull_mode = None;
                base.depth_stencil = true;
                Arc::new(base)
            }
        }
    }
}
