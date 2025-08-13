use image::imageops::FilterType::CatmullRom;
use wgpu::{NoopBackendOptions, PolygonMode, PrimitiveTopology};

use crate::resources::{MaterialShaderDescriptor, ShaderSource, VertexStageLayout};

pub struct DebugMaterialShader;

impl Into<MaterialShaderDescriptor> for DebugMaterialShader {
    fn into(self) -> MaterialShaderDescriptor {
        MaterialShaderDescriptor {
            name: Some("Debug Material Shader".to_string()),
            shader_source: ShaderSource::Path("Assets/Shaders/old/wireframe.wgsl"),
            variables: Vec::new(),
            entrypoint_vertex: "entrypoint_vertex",
            entrypoint_fragment: "entrypoint_fragment",
            vertex_stage_layouts: Some(vec![
                VertexStageLayout::SimpleVertexData,
                VertexStageLayout::InstanceData,
            ]),
            primitive_topology: PrimitiveTopology::TriangleList,
            front_face_order: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: PolygonMode::Line,
            depth_stencil: true,
        }
    }
}
