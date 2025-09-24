use wgpu::{PolygonMode, PrimitiveTopology};

use crate::resources::{MaterialShaderDescriptor, ShaderSource, VertexStageLayout};

pub struct DebugMaterialShader;

impl From<DebugMaterialShader> for MaterialShaderDescriptor {
    fn from(val: DebugMaterialShader) -> Self {
        MaterialShaderDescriptor {
            name: Some("Debug Material Shader".to_string()),
            shader_source: ShaderSource::Path("Assets/Shaders/wireframe.wgsl"),
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
