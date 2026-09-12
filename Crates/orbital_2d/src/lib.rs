//! # Orbital 2D
//!
//! 2D rendering primitives for the Orbital engine.
//!
//! Provides:
//! - [`Camera2D`] — orthographic and perspective cameras for 2D rendering
//! - [`ShapeDescriptor`] — edge-count based shape generation (triangle, quad, circle, polygon)
//! - [`Vertex2D`] — vertex format for 2D rendering
//! - [`Batch2D`] — batched rendering for efficient draw calls
//!
//! ## Quick Start
//!
//! ```ignore
//! use orbital_2d::{Camera2D, ShapeDescriptor, Vertex2D, Batch2D};
//!
//! // Create a camera
//! let camera = Camera2D::orthographic(1.0);
//!
//! // Generate shape vertices
//! let shape = ShapeDescriptor::solid_quad([1.0, 0.0, 0.0, 1.0]);
//! let vertices = orbital_2d::shape::generate_shape_vertices(&shape, 100.0, 100.0);
//!
//! // Create a batch
//! let mut batch = Batch2D::new();
//! batch.push_shape(&vertices);
//! ```

pub mod batch;
pub mod camera;
pub mod shape;
pub mod vertex;

pub use batch::Batch2D;
pub use camera::Camera2D;
pub use shape::{FillMode, ShapeDescriptor};
pub use vertex::Vertex2D;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_and_shape_integration() {
        let camera = Camera2D::orthographic(1.0);
        let shape = ShapeDescriptor::solid_quad([1.0, 0.0, 0.0, 1.0]);
        let vertices = shape::generate_shape_vertices(&shape, 100.0, 100.0);

        let mut batch = Batch2D::new();
        batch.push_shape(&vertices);

        assert_eq!(batch.vertex_count(), 6);

        // Transform vertices using camera
        let mvp = camera.build_view_projection_matrix(800.0, 600.0);
        for v in &batch.vertices {
            let clip = mvp * cgmath::Vector4::new(v.position[0], v.position[1], 0.0, 1.0);
            assert!(clip.x.is_finite());
            assert!(clip.y.is_finite());
        }
    }

    #[test]
    fn batch_rendering_pipeline() {
        let mut batch = Batch2D::new();

        // Add a quad
        let quad = ShapeDescriptor::solid_quad([1.0, 0.0, 0.0, 1.0]);
        batch.push_shape(&shape::generate_shape_vertices(&quad, 50.0, 50.0));

        // Add a circle
        batch.push_shape(&shape::generate_circle([0.0, 0.0], 25.0, 16, [0.0, 1.0, 0.0, 1.0]));

        // Add a triangle
        let tri = ShapeDescriptor::solid_triangle([0.0, 0.0, 1.0, 1.0]);
        batch.push_shape(&shape::generate_shape_vertices(&tri, 40.0, 40.0));

        assert_eq!(batch.draw_call_count(), 3);
        assert_eq!(batch.vertex_count(), 6 + 48 + 3); // quad + circle(16*3) + triangle
    }
}
