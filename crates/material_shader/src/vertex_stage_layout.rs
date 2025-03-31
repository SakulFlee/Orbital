use instance::Instance;
use vertex::Vertex;
use wgpu::VertexBufferLayout;

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
