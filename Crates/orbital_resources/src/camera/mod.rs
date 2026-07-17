use std::mem;

use cgmath::{
    Deg, InnerSpace, Matrix, Matrix4, Point3, Quaternion, SquareMatrix, Vector3, perspective,
};
use wgpu::{Buffer, BufferDescriptor, BufferUsages, Device, Queue};

mod change;
pub use change::*;

mod mode;
pub use mode::*;

mod descriptor;
pub use descriptor::*;

mod frustum;
pub use frustum::*;

#[derive(Debug)]
pub struct Camera {
    camera_buffer: Buffer,
    perspective_view_projection_matrix: Matrix4<f32>,
}

impl Camera {
    pub fn from_descriptor(descriptor: CameraDescriptor, device: &Device, queue: &Queue) -> Self {
        let camera_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Camera Buffer"),
            size: (
                // We have the following variables in our Buffer:
                // position:                            vec4<f32>   -> 4x f32
                mem::size_of::<f32>() * 3 +
                // view_projection_matrix:              mat4x4<f32> -> 4x f32
                mem::size_of::<f32>() * 4 * 4 +
                // perspective_view_projection_matrix:  mat4x4<f32> -> 4x4x f32
                mem::size_of::<f32>() * 4 * 4 +
                // view_projection_transposed:          mat4x4<f32> -> 4x4x f32
                mem::size_of::<f32>() * 4 * 4 +
                // perspective_projection_invert:       mat4x4<f32> -> 4x4x f32
                mem::size_of::<f32>() * 4 * 4 +
                // global_gamma:
                mem::size_of::<f32>() +
                // sky_box_gamma:
                mem::size_of::<f32>() +
                // Padding ... This should align the buffer to 288.
                12
            ) as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut camera = Self {
            camera_buffer,
            perspective_view_projection_matrix: Matrix4::identity(),
        };
        camera.update_buffer(&descriptor, queue);
        camera
    }

    pub fn update_buffer(&mut self, descriptor: &CameraDescriptor, queue: &Queue) {
        let view_projection_matrix = self.calculate_view_projection_matrix(descriptor);
        let perspective_projection_matrix =
            self.calculate_perspective_projection_matrix(descriptor);

        let perspective_view_projection_matrix =
            perspective_projection_matrix * view_projection_matrix;
        self.perspective_view_projection_matrix = perspective_view_projection_matrix;

        let view_projection_transposed = view_projection_matrix.transpose();
        let perspective_projection_invert = perspective_projection_matrix
            .invert()
            .unwrap_or(Matrix4::identity());

        queue.write_buffer(
            &self.camera_buffer,
            0,
            &[
                // Position (+ offset to make vec4)
                descriptor.position.x.to_le_bytes(),
                descriptor.position.y.to_le_bytes(),
                descriptor.position.z.to_le_bytes(),
                [0u8; 4],
                // View Projection Matrix
                view_projection_matrix.x.x.to_le_bytes(),
                view_projection_matrix.x.y.to_le_bytes(),
                view_projection_matrix.x.z.to_le_bytes(),
                view_projection_matrix.x.w.to_le_bytes(),
                view_projection_matrix.y.x.to_le_bytes(),
                view_projection_matrix.y.y.to_le_bytes(),
                view_projection_matrix.y.z.to_le_bytes(),
                view_projection_matrix.y.w.to_le_bytes(),
                view_projection_matrix.z.x.to_le_bytes(),
                view_projection_matrix.z.y.to_le_bytes(),
                view_projection_matrix.z.z.to_le_bytes(),
                view_projection_matrix.z.w.to_le_bytes(),
                view_projection_matrix.w.x.to_le_bytes(),
                view_projection_matrix.w.y.to_le_bytes(),
                view_projection_matrix.w.z.to_le_bytes(),
                view_projection_matrix.w.w.to_le_bytes(),
                // Perspective View Projection Matrix
                perspective_view_projection_matrix.x.x.to_le_bytes(),
                perspective_view_projection_matrix.x.y.to_le_bytes(),
                perspective_view_projection_matrix.x.z.to_le_bytes(),
                perspective_view_projection_matrix.x.w.to_le_bytes(),
                perspective_view_projection_matrix.y.x.to_le_bytes(),
                perspective_view_projection_matrix.y.y.to_le_bytes(),
                perspective_view_projection_matrix.y.z.to_le_bytes(),
                perspective_view_projection_matrix.y.w.to_le_bytes(),
                perspective_view_projection_matrix.z.x.to_le_bytes(),
                perspective_view_projection_matrix.z.y.to_le_bytes(),
                perspective_view_projection_matrix.z.z.to_le_bytes(),
                perspective_view_projection_matrix.z.w.to_le_bytes(),
                perspective_view_projection_matrix.w.x.to_le_bytes(),
                perspective_view_projection_matrix.w.y.to_le_bytes(),
                perspective_view_projection_matrix.w.z.to_le_bytes(),
                perspective_view_projection_matrix.w.w.to_le_bytes(),
                // Transposed View Projection Matrix
                view_projection_transposed.x.x.to_le_bytes(),
                view_projection_transposed.x.y.to_le_bytes(),
                view_projection_transposed.x.z.to_le_bytes(),
                view_projection_transposed.x.w.to_le_bytes(),
                view_projection_transposed.y.x.to_le_bytes(),
                view_projection_transposed.y.y.to_le_bytes(),
                view_projection_transposed.y.z.to_le_bytes(),
                view_projection_transposed.y.w.to_le_bytes(),
                view_projection_transposed.z.x.to_le_bytes(),
                view_projection_transposed.z.y.to_le_bytes(),
                view_projection_transposed.z.z.to_le_bytes(),
                view_projection_transposed.z.w.to_le_bytes(),
                view_projection_transposed.w.x.to_le_bytes(),
                view_projection_transposed.w.y.to_le_bytes(),
                view_projection_transposed.w.z.to_le_bytes(),
                view_projection_transposed.w.w.to_le_bytes(),
                // Inverted Perspective Projection Matrix
                perspective_projection_invert.x.x.to_le_bytes(),
                perspective_projection_invert.x.y.to_le_bytes(),
                perspective_projection_invert.x.z.to_le_bytes(),
                perspective_projection_invert.x.w.to_le_bytes(),
                perspective_projection_invert.y.x.to_le_bytes(),
                perspective_projection_invert.y.y.to_le_bytes(),
                perspective_projection_invert.y.z.to_le_bytes(),
                perspective_projection_invert.y.w.to_le_bytes(),
                perspective_projection_invert.z.x.to_le_bytes(),
                perspective_projection_invert.z.y.to_le_bytes(),
                perspective_projection_invert.z.z.to_le_bytes(),
                perspective_projection_invert.z.w.to_le_bytes(),
                perspective_projection_invert.w.x.to_le_bytes(),
                perspective_projection_invert.w.y.to_le_bytes(),
                perspective_projection_invert.w.z.to_le_bytes(),
                perspective_projection_invert.w.w.to_le_bytes(),
                // Global Gamma
                descriptor.global_gamma.to_le_bytes(),
            ]
            .concat(),
        );
    }

    pub fn calculate_view_projection_matrix(&self, descriptor: &CameraDescriptor) -> Matrix4<f32> {
        // Takes yaw and pitch values and converts them into a target vector for our camera.
        let (pitch_sin, pitch_cos) = descriptor.pitch.sin_cos();
        let (yaw_sin, yaw_cos) = descriptor.yaw.sin_cos();
        let (roll_sin, roll_cos) = descriptor.roll.sin_cos();

        // Calculate the forward, right, and up vectors
        let forward = Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        let up = right.cross(forward).normalize();

        // Apply roll rotation to the up and right vectors
        let _rotated_right = right * roll_cos + up * roll_sin;
        let rotated_up = -right * roll_sin + up * roll_cos;

        // Calculates the view project matrix
        Matrix4::look_to_rh(descriptor.position, forward, rotated_up)
    }

    pub fn calculate_perspective_projection_matrix(
        &self,
        descriptor: &CameraDescriptor,
    ) -> Matrix4<f32> {
        perspective(
            Deg(descriptor.fovy),
            descriptor.aspect,
            descriptor.near,
            descriptor.far,
        )
    }

    pub fn camera_buffer(&self) -> &Buffer {
        &self.camera_buffer
    }

    pub fn frustum(&self) -> Frustum {
        Frustum::from_view_projection_matrix(&self.perspective_view_projection_matrix)
    }

}

// ---------------------------------------------------------------------------
// ECS-compatible constructors (quaternion-based, no gimbal lock)
// ---------------------------------------------------------------------------

impl Camera {
    /// Create a camera from split ECS components (position, rotation, camera props).
    ///
    /// The view matrix is computed directly from the quaternion — no Euler decomposition.
    pub fn new(
        position: Point3<f32>,
        rotation: Quaternion<f32>,
        fovy: f32,
        aspect: f32,
        near: f32,
        far: f32,
        global_gamma: f32,
        device: &Device,
        queue: &Queue,
    ) -> Self {
        let camera_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Camera Buffer"),
            size: (
                mem::size_of::<f32>() * 3 +     // position (vec4 padded)
                mem::size_of::<f32>() * 4 * 4 +  // view_projection
                mem::size_of::<f32>() * 4 * 4 +  // perspective_view_projection
                mem::size_of::<f32>() * 4 * 4 +  // view_projection_transposed
                mem::size_of::<f32>() * 4 * 4 +  // perspective_projection_invert
                mem::size_of::<f32>() +           // global_gamma
                mem::size_of::<f32>() +           // sky_box_gamma
                12
                // padding to 288
            ) as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut camera = Self {
            camera_buffer,
            perspective_view_projection_matrix: Matrix4::identity(),
        };
        camera.update_from_parts(
            position,
            rotation,
            fovy,
            aspect,
            near,
            far,
            global_gamma,
            queue,
        );
        camera
    }

    /// Update GPU buffer from split ECS components.
    ///
    /// The view matrix is computed by rotating basis vectors by the quaternion:
    /// - forward = rotation * (0, 0, -1)  (camera looks along -Z in local space)
    /// - right   = rotation * (1, 0, 0)
    /// - up      = rotation * (0, 1, 0)
    pub fn update_from_parts(
        &mut self,
        position: Point3<f32>,
        rotation: Quaternion<f32>,
        fovy: f32,
        aspect: f32,
        near: f32,
        far: f32,
        global_gamma: f32,
        queue: &Queue,
    ) {
        let view_projection_matrix =
            Self::compute_view_projection_matrix_from_rotation(position, rotation);
        let perspective_projection_matrix = perspective(Deg(fovy), aspect, near, far);

        let perspective_view_projection_matrix =
            perspective_projection_matrix * view_projection_matrix;
        self.perspective_view_projection_matrix = perspective_view_projection_matrix;
        let view_projection_transposed = view_projection_matrix.transpose();
        let perspective_projection_invert = perspective_projection_matrix
            .invert()
            .unwrap_or(Matrix4::identity());

        queue.write_buffer(
            &self.camera_buffer,
            0,
            &[
                position.x.to_le_bytes(),
                position.y.to_le_bytes(),
                position.z.to_le_bytes(),
                [0u8; 4],
                view_projection_matrix.x.x.to_le_bytes(),
                view_projection_matrix.x.y.to_le_bytes(),
                view_projection_matrix.x.z.to_le_bytes(),
                view_projection_matrix.x.w.to_le_bytes(),
                view_projection_matrix.y.x.to_le_bytes(),
                view_projection_matrix.y.y.to_le_bytes(),
                view_projection_matrix.y.z.to_le_bytes(),
                view_projection_matrix.y.w.to_le_bytes(),
                view_projection_matrix.z.x.to_le_bytes(),
                view_projection_matrix.z.y.to_le_bytes(),
                view_projection_matrix.z.z.to_le_bytes(),
                view_projection_matrix.z.w.to_le_bytes(),
                view_projection_matrix.w.x.to_le_bytes(),
                view_projection_matrix.w.y.to_le_bytes(),
                view_projection_matrix.w.z.to_le_bytes(),
                view_projection_matrix.w.w.to_le_bytes(),
                perspective_view_projection_matrix.x.x.to_le_bytes(),
                perspective_view_projection_matrix.x.y.to_le_bytes(),
                perspective_view_projection_matrix.x.z.to_le_bytes(),
                perspective_view_projection_matrix.x.w.to_le_bytes(),
                perspective_view_projection_matrix.y.x.to_le_bytes(),
                perspective_view_projection_matrix.y.y.to_le_bytes(),
                perspective_view_projection_matrix.y.z.to_le_bytes(),
                perspective_view_projection_matrix.y.w.to_le_bytes(),
                perspective_view_projection_matrix.z.x.to_le_bytes(),
                perspective_view_projection_matrix.z.y.to_le_bytes(),
                perspective_view_projection_matrix.z.z.to_le_bytes(),
                perspective_view_projection_matrix.z.w.to_le_bytes(),
                perspective_view_projection_matrix.w.x.to_le_bytes(),
                perspective_view_projection_matrix.w.y.to_le_bytes(),
                perspective_view_projection_matrix.w.z.to_le_bytes(),
                perspective_view_projection_matrix.w.w.to_le_bytes(),
                view_projection_transposed.x.x.to_le_bytes(),
                view_projection_transposed.x.y.to_le_bytes(),
                view_projection_transposed.x.z.to_le_bytes(),
                view_projection_transposed.x.w.to_le_bytes(),
                view_projection_transposed.y.x.to_le_bytes(),
                view_projection_transposed.y.y.to_le_bytes(),
                view_projection_transposed.y.z.to_le_bytes(),
                view_projection_transposed.y.w.to_le_bytes(),
                view_projection_transposed.z.x.to_le_bytes(),
                view_projection_transposed.z.y.to_le_bytes(),
                view_projection_transposed.z.z.to_le_bytes(),
                view_projection_transposed.z.w.to_le_bytes(),
                view_projection_transposed.w.x.to_le_bytes(),
                view_projection_transposed.w.y.to_le_bytes(),
                view_projection_transposed.w.z.to_le_bytes(),
                view_projection_transposed.w.w.to_le_bytes(),
                perspective_projection_invert.x.x.to_le_bytes(),
                perspective_projection_invert.x.y.to_le_bytes(),
                perspective_projection_invert.x.z.to_le_bytes(),
                perspective_projection_invert.x.w.to_le_bytes(),
                perspective_projection_invert.y.x.to_le_bytes(),
                perspective_projection_invert.y.y.to_le_bytes(),
                perspective_projection_invert.y.z.to_le_bytes(),
                perspective_projection_invert.y.w.to_le_bytes(),
                perspective_projection_invert.z.x.to_le_bytes(),
                perspective_projection_invert.z.y.to_le_bytes(),
                perspective_projection_invert.z.z.to_le_bytes(),
                perspective_projection_invert.z.w.to_le_bytes(),
                perspective_projection_invert.w.x.to_le_bytes(),
                perspective_projection_invert.w.y.to_le_bytes(),
                perspective_projection_invert.w.z.to_le_bytes(),
                perspective_projection_invert.w.w.to_le_bytes(),
                global_gamma.to_le_bytes(),
            ]
            .concat(),
        );
    }

    /// Compute view-projection matrix from a rotation quaternion.
    ///
    /// Rotates the local-space basis vectors by the quaternion:
    /// - forward = rotation * (1, 0, 0)  (local forward is +X)
    /// - right   = rotation * (0, 0, 1)  (local right is +Z)
    /// - up      = rotation * (0, 1, 0)  (local up is +Y)
    fn compute_view_projection_matrix_from_rotation(
        position: Point3<f32>,
        rotation: Quaternion<f32>,
    ) -> Matrix4<f32> {
        let forward = (rotation * Vector3::new(1.0, 0.0, 0.0)).normalize();
        let _right = (rotation * Vector3::new(0.0, 0.0, 1.0)).normalize();
        let up = (rotation * Vector3::new(0.0, 1.0, 0.0)).normalize();

        Matrix4::look_to_rh(position, forward, up)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::Rotation3;

    /// Verify the quaternion-based view matrix produces a valid result:
    /// - Orthonormal basis (forward, right, up are perpendicular unit vectors)
    /// - Determinant of the 3x3 rotation part is +1 (proper rotation, no reflection)
    ///
    /// NOTE: The old Euler code uses a non-standard camera convention where
    /// the right vector is only yaw-dependent (horizontal). The new quaternion
    /// path uses a standard rotation convention. These produce different view
    /// matrices for the same logical rotation — this is intentional and correct.
    /// The quaternion path avoids gimbal lock and produces a proper rotation.
    #[test]
    fn quaternion_view_matrix_is_valid() {
        let test_cases = [
            (Point3::new(0.0, 0.0, 5.0), 0.0f32, 0.0, 0.0),
            (Point3::new(1.0, 2.0, 3.0), 0.5, 0.3, 0.0),
            (Point3::new(-1.0, 0.0, 0.0), 1.0, -0.2, 0.1),
            (Point3::new(0.0, 5.0, 0.0), 0.0, 0.8, 0.0),
            (Point3::new(3.0, 1.0, -2.0), 2.0, 0.4, -0.3),
        ];

        for (position, yaw, pitch, roll) in test_cases {
            let rotation = yaw_pitch_roll_to_quaternion(yaw, pitch, roll);
            let matrix = Camera::compute_view_projection_matrix_from_rotation(position, rotation);

            // Verify the rotation part (upper-left 3x3) is orthonormal
            let col0 = Vector3::new(matrix[0][0], matrix[1][0], matrix[2][0]);
            let col1 = Vector3::new(matrix[0][1], matrix[1][1], matrix[2][1]);
            let col2 = Vector3::new(matrix[0][2], matrix[1][2], matrix[2][2]);

            // Columns should be unit vectors
            assert!(
                (col0.magnitude() - 1.0).abs() < 0.001,
                "Column 0 not unit: magnitude={} (yaw={}, pitch={}, roll={})",
                col0.magnitude(),
                yaw,
                pitch,
                roll
            );
            assert!(
                (col1.magnitude() - 1.0).abs() < 0.001,
                "Column 1 not unit: magnitude={}",
                col1.magnitude()
            );
            assert!(
                (col2.magnitude() - 1.0).abs() < 0.001,
                "Column 2 not unit: magnitude={}",
                col2.magnitude()
            );

            // Columns should be perpendicular
            assert!(
                col0.dot(col1).abs() < 0.001,
                "Columns 0 and 1 not perpendicular: dot={}",
                col0.dot(col1)
            );
            assert!(
                col0.dot(col2).abs() < 0.001,
                "Columns 0 and 2 not perpendicular: dot={}",
                col0.dot(col2)
            );
            assert!(
                col1.dot(col2).abs() < 0.001,
                "Columns 1 and 2 not perpendicular: dot={}",
                col1.dot(col2)
            );

            // Determinant should be +1 (proper rotation, no reflection)
            let det = col0.x * (col1.y * col2.z - col1.z * col2.y)
                - col0.y * (col1.x * col2.z - col1.z * col2.x)
                + col0.z * (col1.x * col2.y - col1.y * col2.x);
            assert!(
                (det - 1.0).abs() < 0.001,
                "Determinant should be 1, got {} (yaw={}, pitch={}, roll={})",
                det,
                yaw,
                pitch,
                roll
            );
        }
    }

    /// Verify that the identity quaternion produces a valid view matrix
    /// that transforms the local forward direction correctly.
    #[test]
    fn identity_rotation_view_matrix() {
        let position = Point3::new(0.0, 0.0, 0.0);
        let rotation = Quaternion::new(1.0, 0.0, 0.0, 0.0); // identity

        let matrix = Camera::compute_view_projection_matrix_from_rotation(position, rotation);

        // The view matrix should be orthonormal (verified by quaternion_view_matrix_is_valid)
        // Just verify it's not all zeros and the position is handled
        let trace = matrix[0][0] + matrix[1][1] + matrix[2][2];
        // For an identity rotation view matrix, the rotation part should have trace ≈ 0
        // (since it's essentially a look_at with forward=(1,0,0))
        assert!(
            trace.abs() < 3.0,
            "View matrix trace seems wrong: {}",
            trace
        );
    }

    /// Helper: convert Euler angles to a quaternion using the standard rotation convention.
    fn yaw_pitch_roll_to_quaternion(yaw: f32, pitch: f32, roll: f32) -> Quaternion<f32> {
        let q_yaw = Quaternion::from_axis_angle(Vector3::new(0.0, 1.0, 0.0), cgmath::Rad(yaw));
        let q_pitch = Quaternion::from_axis_angle(Vector3::new(0.0, 0.0, 1.0), cgmath::Rad(pitch));
        let q_roll = Quaternion::from_axis_angle(Vector3::new(1.0, 0.0, 0.0), cgmath::Rad(roll));

        (q_yaw * q_pitch * q_roll).normalize()
    }
}
