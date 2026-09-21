/// Vertex format for 2D rendering.
///
/// Each vertex contains:
/// - Position in world space (2D)
/// - Color (RGBA)
/// - Texture coordinates (UV)
/// - Shape parameters (edge count, reserved for future use)
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Vertex2D {
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub texcoord: [f32; 2],
    pub shape_params: [f32; 2],
}

impl Vertex2D {
    /// Creates a new vertex with position and color.
    pub fn new(position: [f32; 2], color: [f32; 4]) -> Self {
        Self {
            position,
            color,
            texcoord: [0.0, 0.0],
            shape_params: [0.0, 0.0],
        }
    }

    /// Creates a new vertex with position, color, and texture coordinates.
    pub fn with_texcoord(position: [f32; 2], color: [f32; 4], texcoord: [f32; 2]) -> Self {
        Self {
            position,
            color,
            texcoord,
            shape_params: [0.0, 0.0],
        }
    }

    /// Creates a new vertex with all fields.
    pub fn full(
        position: [f32; 2],
        color: [f32; 4],
        texcoord: [f32; 2],
        shape_params: [f32; 2],
    ) -> Self {
        Self {
            position,
            color,
            texcoord,
            shape_params,
        }
    }

    /// Returns the byte size of a single vertex.
    pub fn byte_size() -> usize {
        std::mem::size_of::<Self>()
    }

    /// Returns the wgpu vertex buffer layout for this vertex format.
    pub fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        use wgpu::{VertexAttribute, VertexStepMode};

        static ATTRIBS: [VertexAttribute; 4] = wgpu::vertex_attr_array![
            0 => Float32x2,   // position
            1 => Float32x4,   // color
            2 => Float32x2,   // texcoord
            3 => Float32x2,   // shape_params
        ];

        wgpu::VertexBufferLayout {
            array_stride: Self::byte_size() as wgpu::BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &ATTRIBS,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertex_size() {
        // 2 (position) + 4 (color) + 2 (texcoord) + 2 (shape_params) = 10 floats
        // 10 * 4 bytes = 40 bytes
        assert_eq!(Vertex2D::byte_size(), 40);
    }

    #[test]
    fn vertex_new() {
        let v = Vertex2D::new([1.0, 2.0], [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(v.position, [1.0, 2.0]);
        assert_eq!(v.color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(v.texcoord, [0.0, 0.0]);
        assert_eq!(v.shape_params, [0.0, 0.0]);
    }

    #[test]
    fn vertex_with_texcoord() {
        let v = Vertex2D::with_texcoord([1.0, 2.0], [1.0, 0.0, 0.0, 1.0], [0.5, 0.5]);
        assert_eq!(v.texcoord, [0.5, 0.5]);
    }

    #[test]
    fn vertex_full() {
        let v = Vertex2D::full([1.0, 2.0], [1.0, 0.0, 0.0, 1.0], [0.5, 0.5], [4.0, 0.0]);
        assert_eq!(v.shape_params, [4.0, 0.0]);
    }
}
