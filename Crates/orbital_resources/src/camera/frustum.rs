use cgmath::{InnerSpace, Matrix4, Point3, Vector3, Vector4};

/// A plane in Hessian normal form: `normal · point + d = 0`.
///
/// The normal points **inward** for frustum planes (toward the contained volume).
#[derive(Debug, Clone, PartialEq)]
pub struct Plane {
    pub normal: Vector3<f32>,
    pub d: f32,
}

impl Plane {
    /// Signed distance from this plane to `point`.
    ///
    /// Returns a positive value when the point is on the side of the inward-pointing normal,
    /// negative when on the opposite side.
    pub fn signed_distance(&self, point: &Point3<f32>) -> f32 {
        self.normal.dot(Vector3::new(point.x, point.y, point.z)) + self.d
    }
}

/// A viewing frustum defined by six planes (left, right, bottom, top, near, far).
///
/// Planes are stored with normals pointing **inward** so that
/// `intersects_sphere` can test against all six in a uniform way.
#[derive(Debug, Clone)]
pub struct Frustum {
    pub planes: [Plane; 6],
}

impl Frustum {
    /// Build the six frustum planes from the combined perspective‑view‑projection matrix
    /// using the Gribb/Hartmann method.
    ///
    /// `cgmath::Matrix4` is column‑major, so the four **rows** must be assembled
    /// from individual component accesses before the standard (`row₃ ± rowₙ`) derivation.
    ///
    /// The projection matrix is assumed to follow the wgpu/WebGPU clip convention
    /// (Z clip ∈ [0, 1]), as produced by [`crate::perspective_wgpu`].
    pub fn from_view_projection_matrix(matrix: &Matrix4<f32>) -> Self {
        // cgmath stores matrices column‑major:
        //   .x = column 0, .y = column 1, .z = column 2, .w = column 3
        // Each column is a Vector4 with components [row0, row1, row2, row3].
        //
        // Extract the four rows for the Gribb/Hartmann method:
        let row0 = Vector4::new(matrix.x.x, matrix.y.x, matrix.z.x, matrix.w.x);
        let row1 = Vector4::new(matrix.x.y, matrix.y.y, matrix.z.y, matrix.w.y);
        let row2 = Vector4::new(matrix.x.z, matrix.y.z, matrix.z.z, matrix.w.z);
        let row3 = Vector4::new(matrix.x.w, matrix.y.w, matrix.z.w, matrix.w.w);

        // In clip space the six faces are:
        //   Left:   x >= -w   →  row0 + row3 ≥ 0
        //   Right:  x <=  w   →  row3 - row0 ≥ 0
        //   Bottom: y >= -w   →  row1 + row3 ≥ 0
        //   Top:    y <=  w   →  row3 - row1 ≥ 0
        //   Near:   z >=  0   →  row2 ≥ 0        (wgpu/WebGPU convention, z ∈ [0, 1])
        //   Far:    z <=  w   →  row3 - row2 ≥ 0
        let planes = [
            Self::plane_from_row(row3 + row0), // Left
            Self::plane_from_row(row3 - row0), // Right
            Self::plane_from_row(row3 + row1), // Bottom
            Self::plane_from_row(row3 - row1), // Top
            Self::plane_from_row(row2),        // Near
            Self::plane_from_row(row3 - row2), // Far
        ];

        Frustum { planes }
    }

    /// Normalise a row‑vector into a `Plane`.
    fn plane_from_row(row: Vector4<f32>) -> Plane {
        let normal = Vector3::new(row.x, row.y, row.z);
        let len = normal.magnitude();
        Plane {
            normal: normal / len,
            d: row.w / len,
        }
    }

    /// Test whether a sphere intersects (or is inside) the frustum.
    ///
    /// Returns `true` if the sphere is at least partially inside the frustum.
    /// Returns `false` if the sphere is fully outside (culled).
    pub fn intersects_sphere(&self, center: &Point3<f32>, radius: f32) -> bool {
        for plane in &self.planes {
            if plane.signed_distance(center) <= -radius {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projection::perspective_wgpu;
    use cgmath::{Deg, Rad};

    fn wgpu_proj(fovy: Deg<f32>, aspect: f32, near: f32, far: f32) -> Matrix4<f32> {
        perspective_wgpu(Rad::from(fovy), aspect, near, far, false)
    }

    /// Verify that points known to be inside/outside a frustum
    /// are correctly classified by `intersects_sphere`.
    ///
    /// Camera at origin, looking along +X, 90° FOV, 1.0 aspect.
    /// In view space the visible range at distance `d` is ±d in both Y and Z.
    /// Frustum planes in world space (see detailed derivation):
    ///   Left:   world_x + world_z ≥ 0
    ///   Right:  world_x - world_z ≥ 0
    ///   Near:   world_x ≥ 0.1
    ///   Far:    world_x ≤ 100.1
    #[test]
    fn basic_frustum_intersection() {
        let view = Matrix4::look_to_rh(
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        );
        let proj = wgpu_proj(Deg(90.0), 1.0, 0.1, 100.0);
        let frustum = Frustum::from_view_projection_matrix(&(proj * view));

        // Inside — center of frustum
        assert!(frustum.intersects_sphere(&Point3::new(5.0, 0.0, 0.0), 0.5));

        // Behind camera (near plane at x ≈ 0.1)
        assert!(!frustum.intersects_sphere(&Point3::new(-5.0, 0.0, 0.0), 0.1));

        // Beyond far plane
        assert!(!frustum.intersects_sphere(&Point3::new(200.0, 0.0, 0.0), 10.0));

        // Beyond left plane (world_x + world_z < 0, with offset for radius)
        // For (1.0, 0.0, -1.5): signed_distance = (1 + (-1.5))/√2 ≈ -0.354
        assert!(!frustum.intersects_sphere(&Point3::new(1.0, 0.0, -1.5), 0.1));

        // Beyond right plane (world_x - world_z < 0, with offset for radius)
        // For (1.0, 0.0, 1.5): signed_distance = (1 - 1.5)/√2 ≈ -0.354
        assert!(!frustum.intersects_sphere(&Point3::new(1.0, 0.0, 1.5), 0.1));

        // Straddling near plane — large sphere centred behind camera
        assert!(frustum.intersects_sphere(&Point3::new(0.0, 0.0, 0.0), 2.0));

        // On the boundary (inside) — sphere centre 0.05 past right plane,
        // but radius 0.1 reaches back into the frustum
        assert!(frustum.intersects_sphere(&Point3::new(1.0, 0.0, 1.05), 0.1));
        // Sphere centre exactly at the right-plane boundary
        assert!(!frustum.intersects_sphere(&Point3::new(1.0, 0.0, 1.2), 0.1));
    }

    /// Frustum with identity rotation (looking along +X as per this engine's convention).
    #[test]
    fn identity_frustum_is_correct() {
        let view = Matrix4::look_to_rh(
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        );
        let proj = wgpu_proj(Deg(90.0), 1.0, 0.1, 100.0);
        let frustum = Frustum::from_view_projection_matrix(&(proj * view));

        // A sphere way out in the visible cone
        assert!(frustum.intersects_sphere(&Point3::new(50.0, 20.0, 0.0), 10.0));

        // A sphere entirely behind the camera
        assert!(!frustum.intersects_sphere(&Point3::new(-50.0, 0.0, 0.0), 10.0));
    }
}
