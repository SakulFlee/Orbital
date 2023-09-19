use wgpu::VertexBufferLayout;

pub trait Vertex {
    fn descriptor() -> VertexBufferLayout<'static>;
}
