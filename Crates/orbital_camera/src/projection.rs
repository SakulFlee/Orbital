//! Camera projection modes and matrix construction.

use cgmath::{Deg, Matrix4, Rad};

use orbital_math::{ortho_wgpu, perspective_wgpu};

/// Selects how a [`CameraDescriptor`](crate::CameraDescriptor) is projected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProjectionMode {
    #[default]
    Perspective,
    Orthographic,
}

/// Fully specified projection parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CameraProjection {
    /// `fovy` is in degrees; `aspect` is width / height.
    Perspective { fovy: f32, aspect: f32 },
    /// `height` is the full vertical view size in world units; `aspect` is
    /// width / height.
    Orthographic { height: f32, aspect: f32 },
}

impl CameraProjection {
    pub fn aspect(self) -> f32 {
        match self {
            Self::Perspective { aspect, .. } | Self::Orthographic { aspect, .. } => aspect,
        }
    }

    pub fn fovy_degrees(self) -> Option<f32> {
        match self {
            Self::Perspective { fovy, .. } => Some(fovy),
            Self::Orthographic { .. } => None,
        }
    }

    pub fn orthographic_height(self) -> Option<f32> {
        match self {
            Self::Perspective { .. } => None,
            Self::Orthographic { height, .. } => Some(height),
        }
    }

    /// Builds a wgpu-convention projection matrix.
    pub fn projection_matrix(self, near: f32, far: f32) -> Matrix4<f32> {
        match self {
            Self::Perspective { fovy, aspect } => {
                perspective_wgpu(Rad::from(Deg(fovy)), aspect, near, far, false)
            }
            Self::Orthographic { height, aspect } => {
                let half_height = (height * 0.5).max(0.0001);
                let half_width = half_height * aspect.max(0.0001);
                ortho_wgpu(
                    -half_width,
                    half_width,
                    -half_height,
                    half_height,
                    near,
                    far,
                    false,
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::Vector4;

    fn ndc(matrix: &Matrix4<f32>, point: Vector4<f32>) -> Vector4<f32> {
        let clip = matrix * point;
        Vector4::new(clip.x / clip.w, clip.y / clip.w, clip.z / clip.w, 1.0)
    }

    #[test]
    fn orthographic_projection_uses_height_and_aspect() {
        let projection = CameraProjection::Orthographic {
            height: 10.0,
            aspect: 2.0,
        };
        let matrix = projection.projection_matrix(0.1, 100.0);

        let top_right_near = ndc(&matrix, Vector4::new(10.0, 5.0, -0.1, 1.0));
        assert!((top_right_near.x - 1.0).abs() < 1e-5);
        assert!((top_right_near.y - 1.0).abs() < 1e-5);
        assert!(top_right_near.z.abs() < 1e-5);

        let bottom_left_far = ndc(&matrix, Vector4::new(-10.0, -5.0, -100.0, 1.0));
        assert!((bottom_left_far.x + 1.0).abs() < 1e-5);
        assert!((bottom_left_far.y + 1.0).abs() < 1e-5);
        assert!((bottom_left_far.z - 1.0).abs() < 1e-5);
    }

    #[test]
    fn perspective_and_orthographic_report_their_parameters() {
        let perspective = CameraProjection::Perspective {
            fovy: 60.0,
            aspect: 1.5,
        };
        assert_eq!(perspective.aspect(), 1.5);
        assert_eq!(perspective.fovy_degrees(), Some(60.0));
        assert_eq!(perspective.orthographic_height(), None);

        let orthographic = CameraProjection::Orthographic {
            height: 8.0,
            aspect: 2.0,
        };
        assert_eq!(orthographic.aspect(), 2.0);
        assert_eq!(orthographic.fovy_degrees(), None);
        assert_eq!(orthographic.orthographic_height(), Some(8.0));
    }
}
