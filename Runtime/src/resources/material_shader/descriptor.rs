use wgpu::{Face, FrontFace, PolygonMode, PrimitiveTopology, ShaderStages};

use crate::resources::{ShaderDescriptor, ShaderSource, VariableType, VertexStageLayout};

pub type MaterialDescriptor = MaterialShaderDescriptor;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct MaterialShaderDescriptor {
    pub name: Option<String>,
    pub shader_source: ShaderSource,
    pub variables: Vec<VariableType>,
    pub entrypoint_vertex: &'static str,
    pub entrypoint_fragment: &'static str,
    pub vertex_stage_layouts: Vec<VertexStageLayout>,
    pub primitive_topology: PrimitiveTopology,
    pub front_face_order: FrontFace,
    pub cull_mode: Option<Face>,
    pub polygon_mode: PolygonMode,
    pub depth_stencil: bool,
}

impl ShaderDescriptor for MaterialShaderDescriptor {
    fn name(&self) -> Option<String> {
        self.name.clone()
    }

    fn source(&self) -> ShaderSource {
        self.shader_source
    }

    fn variables(&self) -> Option<&Vec<VariableType>> {
        Some(&self.variables)
    }

    fn stages(&self) -> ShaderStages {
        ShaderStages::VERTEX_FRAGMENT
    }
}

impl Default for MaterialShaderDescriptor {
    fn default() -> Self {
        Self {
            name: Some("Default Material Shader".to_string()),
            shader_source: ShaderSource::default(),
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
