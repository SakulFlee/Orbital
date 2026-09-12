use cgmath::{EuclideanSpace, Matrix4, Point2, Point3, Rad, Vector3};

/// A 2D camera that can project 2D world coordinates to clip space.
///
/// Supports both orthographic (standard 2D) and perspective (2.5D/isometric) projections.
#[derive(Debug, Clone)]
pub enum Camera2D {
    /// Standard 2D orthographic projection.
    /// Coordinates map directly to screen space with zoom support.
    Orthographic {
        /// Center of the camera view in world space.
        eye: Point2<f32>,
        /// Zoom factor. 1.0 = default, 2.0 = zoomed in 2x.
        zoom: f32,
        /// Near clipping plane (default: -1.0).
        near: f32,
        /// Far clipping plane (default: 1.0).
        far: f32,
    },
    /// Perspective projection for 2.5D or isometric views.
    Perspective {
        /// Camera position in 3D space.
        eye: Point3<f32>,
        /// Field of view in radians.
        fov: Rad<f32>,
        /// Near clipping plane.
        near: f32,
        /// Far clipping plane.
        far: f32,
    },
}

impl Default for Camera2D {
    fn default() -> Self {
        Self::Orthographic {
            eye: Point2::origin(),
            zoom: 1.0,
            near: -1.0,
            far: 1.0,
        }
    }
}

impl Camera2D {
    /// Creates a new orthographic camera centered at the origin.
    pub fn orthographic(zoom: f32) -> Self {
        Self::Orthographic {
            eye: Point2::origin(),
            zoom,
            near: -1.0,
            far: 1.0,
        }
    }

    /// Creates a new orthographic camera with a specific center.
    pub fn orthographic_at(eye: Point2<f32>, zoom: f32) -> Self {
        Self::Orthographic {
            eye,
            zoom,
            near: -1.0,
            far: 1.0,
        }
    }

    /// Creates a new perspective camera.
    pub fn perspective(eye: Point3<f32>, fov: Rad<f32>, near: f32, far: f32) -> Self {
        Self::Perspective { eye, fov, near, far }
    }

    /// Builds the view-projection matrix for the given screen size.
    ///
    /// The resulting matrix transforms world-space 2D coordinates to clip space.
    /// For orthographic, this maps (eye - screen/zoom) to (eye + screen/zoom).
    /// For perspective, this uses standard perspective projection.
    pub fn build_view_projection_matrix(&self, screen_width: f32, screen_height: f32) -> Matrix4<f32> {
        match *self {
            Camera2D::Orthographic { eye, zoom, near, far } => {
                let half_width = screen_width / (2.0 * zoom);
                let half_height = screen_height / (2.0 * zoom);

                // Orthographic projection: maps [left, right] x [bottom, top] x [near, far] to NDC
                let left = eye.x - half_width;
                let right = eye.x + half_width;
                let bottom = eye.y - half_height;
                let top = eye.y + half_height;

                cgmath::ortho(left, right, bottom, top, near, far)
            }
            Camera2D::Perspective { eye, fov, near, far } => {
                let aspect = screen_width / screen_height;
                let proj = cgmath::perspective(fov, aspect, near, far);
                let view = Matrix4::look_at_rh(
                    eye,
                    eye + Vector3::new(0.0, 0.0, -1.0), // Look along -Z
                    Vector3::new(0.0, 1.0, 0.0),
                );
                proj * view
            }
        }
    }

    /// Transforms a world-space 2D point to screen-space pixel coordinates.
    pub fn world_to_screen(
        &self,
        world_pos: Point2<f32>,
        screen_width: f32,
        screen_height: f32,
    ) -> Point2<f32> {
        match *self {
            Camera2D::Orthographic { eye, zoom, .. } => {
                let screen_x = (world_pos.x - eye.x) * zoom + screen_width / 2.0;
                let screen_y = (world_pos.y - eye.y) * zoom + screen_height / 2.0;
                Point2::new(screen_x, screen_y)
            }
            Camera2D::Perspective { .. } => {
                // For perspective, we'd need the full VP matrix
                // For now, return a simple approximation
                Point2::new(
                    world_pos.x + screen_width / 2.0,
                    world_pos.y + screen_height / 2.0,
                )
            }
        }
    }

    /// Transforms a screen-space pixel coordinate to world-space 2D point.
    pub fn screen_to_world(
        &self,
        screen_pos: Point2<f32>,
        screen_width: f32,
        screen_height: f32,
    ) -> Point2<f32> {
        match *self {
            Camera2D::Orthographic { eye, zoom, .. } => {
                let world_x = (screen_pos.x - screen_width / 2.0) / zoom + eye.x;
                let world_y = (screen_pos.y - screen_height / 2.0) / zoom + eye.y;
                Point2::new(world_x, world_y)
            }
            Camera2D::Perspective { .. } => {
                // For perspective, we'd need the inverse VP matrix
                // For now, return a simple approximation
                Point2::new(
                    screen_pos.x - screen_width / 2.0,
                    screen_pos.y - screen_height / 2.0,
                )
            }
        }
    }

    /// Returns the eye position (center of view).
    pub fn eye(&self) -> Point2<f32> {
        match *self {
            Camera2D::Orthographic { eye, .. } => eye,
            Camera2D::Perspective { eye, .. } => Point2::new(eye.x, eye.y),
        }
    }

    /// Returns the zoom factor (1.0 for perspective cameras).
    pub fn zoom(&self) -> f32 {
        match *self {
            Camera2D::Orthographic { zoom, .. } => zoom,
            Camera2D::Perspective { .. } => 1.0,
        }
    }

    /// Moves the camera by the given offset in world space.
    pub fn translate(&mut self, offset: Vector3<f32>) {
        match self {
            Camera2D::Orthographic { eye, .. } => {
                eye.x += offset.x;
                eye.y += offset.y;
            }
            Camera2D::Perspective { eye, .. } => {
                eye.x += offset.x;
                eye.y += offset.y;
                eye.z += offset.z;
            }
        }
    }

    /// Sets the zoom factor (only affects orthographic cameras).
    pub fn set_zoom(&mut self, zoom: f32) {
        if let Camera2D::Orthographic { zoom: z, .. } = self {
            *z = zoom;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orthographic_default() {
        let cam = Camera2D::default();
        assert_eq!(cam.eye(), Point2::origin());
        assert_eq!(cam.zoom(), 1.0);
    }

    #[test]
    fn orthographic_zoom() {
        let cam = Camera2D::orthographic(2.0);
        assert_eq!(cam.zoom(), 2.0);
    }

    #[test]
    fn orthographic_view_projection() {
        let cam = Camera2D::orthographic(1.0);
        let mvp = cam.build_view_projection_matrix(100.0, 100.0);

        // Identity-like for orthographic at origin with zoom=1
        // Screen center (50, 50) should map to NDC (0, 0)
        let screen_center = mvp * cgmath::Vector4::new(0.0, 0.0, 0.0, 1.0);
        assert!((screen_center.x).abs() < 0.01);
        assert!((screen_center.y).abs() < 0.01);
    }

    #[test]
    fn world_to_screen_conversion() {
        let cam = Camera2D::orthographic(1.0);
        let screen = cam.world_to_screen(Point2::new(10.0, 20.0), 100.0, 100.0);
        assert_eq!(screen, Point2::new(60.0, 70.0));
    }

    #[test]
    fn screen_to_world_conversion() {
        let cam = Camera2D::orthographic(1.0);
        let world = cam.screen_to_world(Point2::new(60.0, 70.0), 100.0, 100.0);
        assert_eq!(world, Point2::new(10.0, 20.0));
    }

    #[test]
    fn translate_camera() {
        let mut cam = Camera2D::orthographic(1.0);
        cam.translate(Vector3::new(10.0, 20.0, 0.0));
        assert_eq!(cam.eye(), Point2::new(10.0, 20.0));
    }

    #[test]
    fn set_zoom() {
        let mut cam = Camera2D::orthographic(1.0);
        cam.set_zoom(2.0);
        assert_eq!(cam.zoom(), 2.0);

        // Perspective cameras ignore zoom
        let mut cam = Camera2D::perspective(
            Point3::new(0.0, 0.0, 5.0),
            Rad(std::f32::consts::FRAC_PI_4),
            0.1,
            100.0,
        );
        cam.set_zoom(2.0);
        assert_eq!(cam.zoom(), 1.0);
    }

    #[test]
    fn roundtrip_world_screen() {
        let cam = Camera2D::orthographic(2.0);
        let world = Point2::new(15.0, 25.0);
        let screen = cam.world_to_screen(world, 800.0, 600.0);
        let back = cam.screen_to_world(screen, 800.0, 600.0);
        assert!((world.x - back.x).abs() < 0.01);
        assert!((world.y - back.y).abs() < 0.01);
    }
}
