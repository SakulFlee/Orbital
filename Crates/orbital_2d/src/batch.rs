use crate::vertex::Vertex2D;

/// A single draw call in a batch.
#[derive(Debug, Clone)]
pub struct DrawCall {
    /// Number of vertices to draw.
    pub vertex_count: u32,
    /// Index into the batch's vertex buffer where this draw starts.
    pub vertex_offset: u32,
}

/// A batch of 2D geometry to be rendered in a single draw call.
///
/// Collects vertices and draw calls for efficient GPU rendering.
#[derive(Debug, Clone)]
pub struct Batch2D {
    /// All vertices in this batch.
    pub vertices: Vec<Vertex2D>,
    /// Draw calls to execute.
    pub draw_calls: Vec<DrawCall>,
}

impl Batch2D {
    /// Creates a new empty batch.
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            draw_calls: Vec::new(),
        }
    }

    /// Creates a new batch with pre-allocated capacity.
    pub fn with_capacity(vertex_capacity: usize, draw_call_capacity: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(vertex_capacity),
            draw_calls: Vec::with_capacity(draw_call_capacity),
        }
    }

    /// Pushes a shape's vertices into the batch.
    pub fn push_shape(&mut self, vertices: &[Vertex2D]) {
        if vertices.is_empty() {
            return;
        }

        let vertex_offset = self.vertices.len() as u32;
        let vertex_count = vertices.len() as u32;

        self.vertices.extend_from_slice(vertices);
        self.draw_calls.push(DrawCall {
            vertex_count,
            vertex_offset,
        });
    }

    /// Pushes multiple shapes into the batch.
    pub fn push_shapes(&mut self, shapes: &[Vec<Vertex2D>]) {
        for shape in shapes {
            self.push_shape(shape);
        }
    }

    /// Returns the total number of vertices in the batch.
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Returns the number of draw calls.
    pub fn draw_call_count(&self) -> usize {
        self.draw_calls.len()
    }

    /// Returns true if the batch is empty.
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    /// Clears the batch for reuse.
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.draw_calls.clear();
    }

    /// Returns a slice of vertices as bytes for GPU upload.
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.vertices.as_ptr() as *const u8,
                self.vertices.len() * std::mem::size_of::<Vertex2D>(),
            )
        }
    }

    /// Merges another batch into this one.
    pub fn merge(&mut self, other: &Batch2D) {
        let vertex_offset = self.vertices.len() as u32;
        self.vertices.extend_from_slice(&other.vertices);

        for draw_call in &other.draw_calls {
            self.draw_calls.push(DrawCall {
                vertex_count: draw_call.vertex_count,
                vertex_offset: draw_call.vertex_offset + vertex_offset,
            });
        }
    }
}

impl Default for Batch2D {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_vertex(x: f32, y: f32) -> Vertex2D {
        Vertex2D::new([x, y], [1.0, 1.0, 1.0, 1.0])
    }

    #[test]
    fn empty_batch() {
        let batch = Batch2D::new();
        assert!(batch.is_empty());
        assert_eq!(batch.vertex_count(), 0);
        assert_eq!(batch.draw_call_count(), 0);
    }

    #[test]
    fn push_single_shape() {
        let mut batch = Batch2D::new();
        let verts = vec![test_vertex(0.0, 0.0), test_vertex(1.0, 0.0), test_vertex(0.5, 1.0)];

        batch.push_shape(&verts);

        assert_eq!(batch.vertex_count(), 3);
        assert_eq!(batch.draw_call_count(), 1);
        assert_eq!(batch.draw_calls[0].vertex_count, 3);
        assert_eq!(batch.draw_calls[0].vertex_offset, 0);
    }

    #[test]
    fn push_multiple_shapes() {
        let mut batch = Batch2D::new();
        let shape1 = vec![test_vertex(0.0, 0.0), test_vertex(1.0, 0.0)];
        let shape2 = vec![test_vertex(0.0, 1.0), test_vertex(1.0, 1.0)];

        batch.push_shape(&shape1);
        batch.push_shape(&shape2);

        assert_eq!(batch.vertex_count(), 4);
        assert_eq!(batch.draw_call_count(), 2);
        assert_eq!(batch.draw_calls[0].vertex_offset, 0);
        assert_eq!(batch.draw_calls[1].vertex_offset, 2);
    }

    #[test]
    fn clear_batch() {
        let mut batch = Batch2D::new();
        batch.push_shape(&vec![test_vertex(0.0, 0.0)]);

        batch.clear();

        assert!(batch.is_empty());
    }

    #[test]
    fn merge_batches() {
        let mut batch1 = Batch2D::new();
        batch1.push_shape(&vec![test_vertex(0.0, 0.0), test_vertex(1.0, 0.0)]);

        let mut batch2 = Batch2D::new();
        batch2.push_shape(&vec![test_vertex(2.0, 0.0), test_vertex(3.0, 0.0)]);

        batch1.merge(&batch2);

        assert_eq!(batch1.vertex_count(), 4);
        assert_eq!(batch1.draw_call_count(), 2);
        // Second batch's draw calls should be offset
        assert_eq!(batch1.draw_calls[1].vertex_offset, 2);
    }

    #[test]
    fn as_bytes_length() {
        let mut batch = Batch2D::new();
        batch.push_shape(&vec![test_vertex(0.0, 0.0)]);

        let bytes = batch.as_bytes();
        assert_eq!(bytes.len(), std::mem::size_of::<Vertex2D>());
    }

    #[test]
    fn push_shapes_macro() {
        let mut batch = Batch2D::new();
        let shapes = vec![
            vec![test_vertex(0.0, 0.0), test_vertex(1.0, 0.0)],
            vec![test_vertex(0.0, 1.0), test_vertex(1.0, 1.0)],
        ];

        batch.push_shapes(&shapes);

        assert_eq!(batch.vertex_count(), 4);
        assert_eq!(batch.draw_call_count(), 2);
    }
}
