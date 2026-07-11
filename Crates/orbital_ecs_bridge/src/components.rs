use cgmath::{InnerSpace, Point3, Quaternion, Rad, Rotation as _, Rotation3, Vector3};

/// World-space position of an entity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position(pub Point3<f32>);

impl Position {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self(Point3::new(x, y, z))
    }

    /// Offset position by a world-space delta.
    pub fn offset(&mut self, delta: Vector3<f32>) {
        self.0 += delta;
    }

    /// Offset position relative to the entity's orientation.
    /// `forward_amount` moves along the entity's forward axis.
    /// `right_amount` moves along the entity's right axis.
    /// `up_amount` moves along the entity's up axis.
    pub fn offset_view_aligned(
        &mut self,
        rotation: &Rotation,
        forward_amount: f32,
        right_amount: f32,
        up_amount: f32,
    ) {
        let (forward, right, up) = rotation.forward_right_up();
        self.0 += forward * forward_amount;
        self.0 += right * right_amount;
        self.0 += up * up_amount;
    }
}

impl Default for Position {
    fn default() -> Self {
        Self(Point3::new(0.0, 0.0, 0.0))
    }
}

/// World-space orientation of an entity, stored as a quaternion.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rotation(pub Quaternion<f32>);

impl Rotation {
    /// Identity rotation (no orientation change).
    pub fn identity() -> Self {
        Self(Quaternion::new(1.0, 0.0, 0.0, 0.0))
    }

    /// Create a rotation that looks toward `target` from the origin,
    /// with the given `up` direction.
    pub fn look_at(target: Point3<f32>, up: Vector3<f32>) -> Self {
        let dir = Vector3::new(target.x, target.y, target.z).normalize();
        Self(Quaternion::look_at(dir, up))
    }

    /// Returns (forward, right, up) unit vectors from the rotation.
    ///
    /// Convention: forward = -Z, right = +X, up = +Y (right-handed, Y-up).
    pub fn forward_right_up(&self) -> (Vector3<f32>, Vector3<f32>, Vector3<f32>) {
        let forward = self.0 * Vector3::new(0.0, 0.0, -1.0);
        let right = self.0 * Vector3::new(1.0, 0.0, 0.0);
        let up = self.0 * Vector3::new(0.0, 1.0, 0.0);
        (forward.normalize(), right.normalize(), up.normalize())
    }

    /// Apply pitch rotation (around the local right axis).
    pub fn rotate_pitch(&mut self, delta: Rad<f32>) {
        let (_, right, _) = self.forward_right_up();
        let rotation = Quaternion::from_axis_angle(right, delta);
        self.0 = (rotation * self.0).normalize();
    }

    /// Apply yaw rotation (around the world Y axis).
    pub fn rotate_yaw(&mut self, delta: Rad<f32>) {
        let rotation = Quaternion::from_axis_angle(Vector3::unit_y(), delta);
        self.0 = (rotation * self.0).normalize();
    }

    /// Apply roll rotation (around the local forward axis).
    pub fn rotate_roll(&mut self, delta: Rad<f32>) {
        let (forward, _, _) = self.forward_right_up();
        let rotation = Quaternion::from_axis_angle(forward, delta);
        self.0 = (rotation * self.0).normalize();
    }
}

impl Default for Rotation {
    fn default() -> Self {
        Self::identity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::{FRAC_PI_2, PI};

    #[test]
    fn identity_rotation() {
        let rot = Rotation::identity();
        let (f, r, u) = rot.forward_right_up();
        assert!((f - Vector3::new(0.0, 0.0, -1.0)).magnitude() < 0.001);
        assert!((r - Vector3::new(1.0, 0.0, 0.0)).magnitude() < 0.001);
        assert!((u - Vector3::new(0.0, 1.0, 0.0)).magnitude() < 0.001);
    }

    #[test]
    fn yaw_90_degrees() {
        let mut rot = Rotation::identity();
        rot.rotate_yaw(Rad(FRAC_PI_2));
        let (f, r, u) = rot.forward_right_up();
        // After 90° yaw, forward should point toward -X (or +X depending on convention)
        assert!(f.x.abs() > 0.9, "forward.x should be near ±1, got {}", f.x);
        assert!(f.y.abs() < 0.01, "forward.y should be near 0, got {}", f.y);
        assert!(f.z.abs() < 0.01, "forward.z should be near 0, got {}", f.z);
        // Up should remain roughly Y-up
        assert!(u.y.abs() > 0.9, "up.y should be near ±1, got {}", u.y);
    }

    #[test]
    fn pitch_90_degrees() {
        let mut rot = Rotation::identity();
        rot.rotate_pitch(Rad(FRAC_PI_2));
        let (f, r, u) = rot.forward_right_up();
        // After 90° pitch up, forward should point toward +Y
        assert!(f.y.abs() > 0.9, "forward.y should be near ±1, got {}", f.y);
        assert!(f.x.abs() < 0.01, "forward.x should be near 0, got {}", f.x);
        // Right should remain roughly X-right
        assert!(r.x.abs() > 0.9, "right.x should be near ±1, got {}", r.x);
    }

    #[test]
    fn position_offset_view_aligned() {
        let mut pos = Position::new(0.0, 0.0, 0.0);
        let rot = Rotation::identity();
        pos.offset_view_aligned(&rot, 1.0, 0.0, 0.0);
        // Forward is -Z for identity, so moving forward 1.0 should go to z=-1.0
        assert!((pos.0.z - (-1.0)).abs() < 0.001, "Expected z=-1.0, got {}", pos.0.z);
    }
}
