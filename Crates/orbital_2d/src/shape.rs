use crate::vertex::Vertex2D;

/// Fill mode for shapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillMode {
    /// Solid color fill.
    Solid,
    /// Textured fill (would use texture coordinates).
    Textured,
}

/// Descriptor for generating a 2D shape.
#[derive(Debug, Clone)]
pub struct ShapeDescriptor {
    /// Number of edges. 3=triangle, 4=quad, 8=octagon, 16+=circle.
    pub edge_count: u32,
    /// RGBA color for the shape.
    pub color: [f32; 4],
    /// Fill mode (solid or textured).
    pub fill: FillMode,
}

impl Default for ShapeDescriptor {
    fn default() -> Self {
        Self {
            edge_count: 4,
            color: [1.0, 1.0, 1.0, 1.0],
            fill: FillMode::Solid,
        }
    }
}

impl ShapeDescriptor {
    pub fn solid_quad(color: [f32; 4]) -> Self {
        Self {
            edge_count: 4,
            color,
            fill: FillMode::Solid,
        }
    }

    pub fn solid_triangle(color: [f32; 4]) -> Self {
        Self {
            edge_count: 3,
            color,
            fill: FillMode::Solid,
        }
    }

    pub fn solid_circle(color: [f32; 4]) -> Self {
        Self {
            edge_count: 32,
            color,
            fill: FillMode::Solid,
        }
    }

    pub fn solid_polygon(edge_count: u32, color: [f32; 4]) -> Self {
        Self {
            edge_count,
            color,
            fill: FillMode::Solid,
        }
    }
}

/// Generates vertices for a 2D shape centered at the origin.
///
/// Returns a list of vertices that can be rendered as a triangle list.
/// For quads (edge_count=4), returns 6 vertices (2 triangles).
/// For other shapes, returns edge_count * 3 vertices (one triangle per edge).
pub fn generate_shape_vertices(
    descriptor: &ShapeDescriptor,
    width: f32,
    height: f32,
) -> Vec<Vertex2D> {
    match descriptor.edge_count {
        3 => generate_triangle(descriptor, width, height),
        4 => generate_quad(descriptor, width, height),
        _ => generate_polygon(descriptor, width, height),
    }
}

/// Generates vertices for a triangle.
fn generate_triangle(desc: &ShapeDescriptor, width: f32, height: f32) -> Vec<Vertex2D> {
    let half_w = width / 2.0;
    let half_h = height / 2.0;
    let color = desc.color;

    // Triangle pointing up
    let vertices = vec![
        Vertex2D::new([0.0, half_h], color),      // Top
        Vertex2D::new([-half_w, -half_h], color), // Bottom-left
        Vertex2D::new([half_w, -half_h], color),  // Bottom-right
    ];

    vertices
}

/// Generates vertices for a quad (2 triangles).
fn generate_quad(desc: &ShapeDescriptor, width: f32, height: f32) -> Vec<Vertex2D> {
    let half_w = width / 2.0;
    let half_h = height / 2.0;
    let color = desc.color;

    // Two triangles forming a quad
    let vertices = vec![
        // Triangle 1
        Vertex2D::new([-half_w, -half_h], color),
        Vertex2D::new([half_w, -half_h], color),
        Vertex2D::new([half_w, half_h], color),
        // Triangle 2
        Vertex2D::new([-half_w, -half_h], color),
        Vertex2D::new([half_w, half_h], color),
        Vertex2D::new([-half_w, half_h], color),
    ];

    vertices
}

/// Generates vertices for a regular polygon with N edges.
fn generate_polygon(desc: &ShapeDescriptor, width: f32, height: f32) -> Vec<Vertex2D> {
    let edge_count = desc.edge_count.max(3);
    let half_w = width / 2.0;
    let half_h = height / 2.0;
    let color = desc.color;

    let mut vertices = Vec::with_capacity(edge_count as usize * 3);

    for i in 0..edge_count {
        let angle_a = i as f32 * std::f32::consts::TAU / edge_count as f32;
        let angle_b = (i + 1) as f32 * std::f32::consts::TAU / edge_count as f32;

        let ax = angle_a.cos() * half_w;
        let ay = angle_a.sin() * half_h;
        let bx = angle_b.cos() * half_w;
        let by = angle_b.sin() * half_h;

        // Center vertex
        vertices.push(Vertex2D::new([0.0, 0.0], color));
        // Edge vertex A
        vertices.push(Vertex2D::new([ax, ay], color));
        // Edge vertex B
        vertices.push(Vertex2D::new([bx, by], color));
    }

    vertices
}

/// Generates vertices for a filled rectangle (axis-aligned quad).
pub fn generate_rect(x: f32, y: f32, width: f32, height: f32, color: [f32; 4]) -> Vec<Vertex2D> {
    vec![
        Vertex2D::new([x, y], color),
        Vertex2D::new([x + width, y], color),
        Vertex2D::new([x + width, y + height], color),
        Vertex2D::new([x, y], color),
        Vertex2D::new([x + width, y + height], color),
        Vertex2D::new([x, y + height], color),
    ]
}

/// Generates vertices for a filled circle using triangle fan.
pub fn generate_circle(
    center: [f32; 2],
    radius: f32,
    segments: u32,
    color: [f32; 4],
) -> Vec<Vertex2D> {
    let mut vertices = Vec::with_capacity(segments as usize * 3);

    for i in 0..segments {
        let angle_a = i as f32 * std::f32::consts::TAU / segments as f32;
        let angle_b = (i + 1) as f32 * std::f32::consts::TAU / segments as f32;

        let ax = center[0] + angle_a.cos() * radius;
        let ay = center[1] + angle_a.sin() * radius;
        let bx = center[0] + angle_b.cos() * radius;
        let by = center[1] + angle_b.sin() * radius;

        vertices.push(Vertex2D::new(center, color));
        vertices.push(Vertex2D::new([ax, ay], color));
        vertices.push(Vertex2D::new([bx, by], color));
    }

    vertices
}

/// Generates indices for a shape (sequential 0, 1, 2, 3, ...).
pub fn generate_indices(vertex_count: u32) -> Vec<u32> {
    (0..vertex_count).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quad_vertices_count() {
        let desc = ShapeDescriptor::solid_quad([1.0, 1.0, 1.0, 1.0]);
        let verts = generate_shape_vertices(&desc, 100.0, 100.0);
        assert_eq!(verts.len(), 6); // 2 triangles * 3 vertices
    }

    #[test]
    fn triangle_vertices_count() {
        let desc = ShapeDescriptor::solid_triangle([1.0, 1.0, 1.0, 1.0]);
        let verts = generate_shape_vertices(&desc, 100.0, 100.0);
        assert_eq!(verts.len(), 3);
    }

    #[test]
    fn polygon_vertices_count() {
        let desc = ShapeDescriptor::solid_polygon(8, [1.0, 1.0, 1.0, 1.0]);
        let verts = generate_shape_vertices(&desc, 100.0, 100.0);
        assert_eq!(verts.len(), 8 * 3); // 8 triangles
    }

    #[test]
    fn circle_vertices_count() {
        let verts = generate_circle([0.0, 0.0], 50.0, 32, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(verts.len(), 32 * 3);
    }

    #[test]
    fn rect_vertices_count() {
        let verts = generate_rect(0.0, 0.0, 100.0, 50.0, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(verts.len(), 6);
    }

    #[test]
    fn indices_sequential() {
        let indices = generate_indices(6);
        assert_eq!(indices, vec![0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn shape_center_at_origin() {
        let desc = ShapeDescriptor::solid_quad([1.0, 1.0, 1.0, 1.0]);
        let verts = generate_shape_vertices(&desc, 100.0, 100.0);
        // All vertices should have coordinates in range [-50, 50]
        for v in &verts {
            assert!(v.position[0].abs() <= 50.01);
            assert!(v.position[1].abs() <= 50.01);
        }
    }

    #[test]
    fn default_shape_is_quad() {
        let desc = ShapeDescriptor::default();
        assert_eq!(desc.edge_count, 4);
    }
}
