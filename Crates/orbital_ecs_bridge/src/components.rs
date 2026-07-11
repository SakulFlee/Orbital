use std::sync::{Arc, RwLock};

use cgmath::{InnerSpace, Point3, Quaternion, Rad, Rotation as _, Rotation3, Vector3};
use orbital_ecs::Entity;
use orbital_resources::Camera;

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
    /// Convention: forward = +X, right = +Z, up = +Y (matching the Camera's
    /// local-space basis for view matrix computation).
    pub fn forward_right_up(&self) -> (Vector3<f32>, Vector3<f32>, Vector3<f32>) {
        let forward = self.0 * Vector3::new(1.0, 0.0, 0.0);
        let right = self.0 * Vector3::new(0.0, 0.0, 1.0);
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

// ---------------------------------------------------------------------------
// Camera ECS types
// ---------------------------------------------------------------------------

/// Camera-only properties (FOV, aspect ratio, clip planes, gamma).
/// Position and rotation come from separate `Position`/`Rotation` components.
#[derive(Debug, Clone)]
pub struct CameraDescriptorEcs {
    pub label: String,
    pub aspect: f32,
    pub fovy: Rad<f32>,
    pub near: f32,
    pub far: f32,
    pub global_gamma: f32,
}

impl Default for CameraDescriptorEcs {
    fn default() -> Self {
        Self {
            label: "Default".into(),
            aspect: 16.0 / 9.0,
            fovy: Rad(std::f32::consts::FRAC_PI_4),
            near: 0.1,
            far: 10000.0,
            global_gamma: 2.2,
        }
    }
}

/// GPU camera state. Shared via `Arc`, mutable via `RwLock`.
/// This is the "realization" link — attaching this to an entity means
/// its GPU representation has been created.
#[derive(Debug, Clone)]
pub struct CameraRealization(pub Arc<RwLock<Camera>>);

/// Marks an entity as the active camera for rendering.
#[derive(Debug, Clone, Copy)]
pub struct ActiveCamera(pub Entity);

/// Dirty flag — set when position/rotation/camera-descriptor change.
/// Cleared by the realization system after GPU buffer update.
#[derive(Debug, Clone, Copy, Default)]
pub struct CameraDirty(pub bool);

impl CameraDirty {
    pub fn is_dirty(&self) -> bool {
        self.0
    }

    pub fn mark_dirty(&mut self) {
        self.0 = true;
    }

    pub fn clear(&mut self) {
        self.0 = false;
    }
}

// ---------------------------------------------------------------------------
// Model ECS types
// ---------------------------------------------------------------------------

use ulid::Ulid;

/// Model-only properties (mesh, materials). Instances are in a separate component.
#[derive(Debug, Clone)]
pub struct ModelDescriptorEcs {
    pub label: String,
    pub mesh: std::sync::Arc<orbital_resources::MeshDescriptor>,
    pub materials: Vec<std::sync::Arc<orbital_resources::MaterialShaderDescriptor>>,
}

impl ModelDescriptorEcs {
    /// Compute a deterministic hash for GPU instancing deduplication.
    pub fn instance_hash(&self) -> Ulid {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.mesh.vertices.hash(&mut hasher);
        self.mesh.indices.hash(&mut hasher);
        for material in &self.materials {
            material.hash(&mut hasher);
        }
        let hash_u64 = hasher.finish();
        let bytes = [
            0, 0, 0, 0, 0, 0,
            (hash_u64 >> 56) as u8,
            (hash_u64 >> 48) as u8,
            (hash_u64 >> 40) as u8,
            (hash_u64 >> 32) as u8,
            (hash_u64 >> 24) as u8,
            (hash_u64 >> 16) as u8,
            (hash_u64 >> 8) as u8,
            hash_u64 as u8,
            0, 0,
        ];
        Ulid::from_bytes(bytes)
    }
}

/// Instance transforms for a model (ULID → Transform mapping).
/// Each entry represents one instance of the model at a different position/rotation/scale.
#[derive(Debug, Clone, Default)]
pub struct ModelInstances(pub hashbrown::HashMap<Ulid, orbital_resources::Transform>);

impl ModelInstances {
    pub fn new() -> Self {
        Self(hashbrown::HashMap::new())
    }

    pub fn add_instance(&mut self, transform: orbital_resources::Transform) -> Ulid {
        let ulid = Ulid::new();
        self.0.insert(ulid, transform);
        ulid
    }

    pub fn remove_instance(&mut self, ulid: &Ulid) -> Option<orbital_resources::Transform> {
        self.0.remove(ulid)
    }
}

/// GPU model state. Shared via `Arc`.
/// This is the "realization" link component.
#[derive(Debug, Clone)]
pub struct ModelRealization(pub std::sync::Arc<orbital_resources::Model>);

/// Dirty flag — set when model descriptor or instances change.
#[derive(Debug, Clone, Copy, Default)]
pub struct ModelDirty(pub bool);

impl ModelDirty {
    pub fn is_dirty(&self) -> bool {
        self.0
    }

    pub fn mark_dirty(&mut self) {
        self.0 = true;
    }

    pub fn clear(&mut self) {
        self.0 = false;
    }
}

// ---------------------------------------------------------------------------
// Light ECS types
// ---------------------------------------------------------------------------

/// Light-only properties (type, color, direction).
/// Position comes from the Position component on the entity.
#[derive(Debug, Clone)]
pub struct LightDescriptorEcs {
    pub light_type: orbital_resources::LightType,
    pub color: cgmath::Vector3<f32>,
    pub direction: cgmath::Vector3<f32>,
}

impl LightDescriptorEcs {
    pub fn new_point(color: cgmath::Vector3<f32>, intensity: f32) -> Self {
        Self {
            light_type: orbital_resources::LightType::Point { intensity },
            color,
            direction: cgmath::Vector3::new(0.0, -1.0, 0.0),
        }
    }

    pub fn new_directional(
        direction: cgmath::Vector3<f32>,
        color: cgmath::Vector3<f32>,
        intensity: f32,
    ) -> Self {
        Self {
            light_type: orbital_resources::LightType::Directional { intensity },
            color,
            direction,
        }
    }

    pub fn new_spot(
        color: cgmath::Vector3<f32>,
        intensity: f32,
        direction: cgmath::Vector3<f32>,
        inner_cone_angle: f32,
        outer_cone_angle: f32,
    ) -> Self {
        Self {
            light_type: orbital_resources::LightType::Spot {
                intensity,
                inner_cone_angle,
                outer_cone_angle,
            },
            color,
            direction,
        }
    }
}

/// Dirty flag — set when light properties change.
#[derive(Debug, Clone, Copy, Default)]
pub struct LightDirty(pub bool);

impl LightDirty {
    pub fn is_dirty(&self) -> bool {
        self.0
    }

    pub fn mark_dirty(&mut self) {
        self.0 = true;
    }

    pub fn clear(&mut self) {
        self.0 = false;
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
        assert!((f - Vector3::new(1.0, 0.0, 0.0)).magnitude() < 0.001);
        assert!((r - Vector3::new(0.0, 0.0, 1.0)).magnitude() < 0.001);
        assert!((u - Vector3::new(0.0, 1.0, 0.0)).magnitude() < 0.001);
    }

    #[test]
    fn yaw_90_degrees() {
        let mut rot = Rotation::identity();
        rot.rotate_yaw(Rad(FRAC_PI_2));
        let (f, r, u) = rot.forward_right_up();
        // After 90° yaw, forward (originally +X) rotates to -Z
        assert!(f.z.abs() > 0.9, "forward.z should be near ±1, got {}", f.z);
        assert!(f.y.abs() < 0.01, "forward.y should be near 0, got {}", f.y);
        // Right (originally +Z) rotates to +X
        assert!(r.x.abs() > 0.9, "right.x should be near ±1, got {}", r.x);
        // Up should remain roughly Y-up
        assert!(u.y.abs() > 0.9, "up.y should be near ±1, got {}", u.y);
    }

    #[test]
    fn pitch_90_degrees() {
        let mut rot = Rotation::identity();
        rot.rotate_pitch(Rad(FRAC_PI_2));
        let (f, r, u) = rot.forward_right_up();
        // After 90° pitch (rotation around +Z), forward (+X) rotates to +Y
        assert!(f.y.abs() > 0.9, "forward.y should be near ±1, got {}", f.y);
        assert!(f.x.abs() < 0.01, "forward.x should be near 0, got {}", f.x);
        // Right (+Z) is unchanged by rotation around Z
        assert!(r.z.abs() > 0.9, "right.z should be near ±1, got {}", r.z);
    }

    #[test]
    fn position_offset_view_aligned() {
        let mut pos = Position::new(0.0, 0.0, 0.0);
        let rot = Rotation::identity();
        pos.offset_view_aligned(&rot, 1.0, 0.0, 0.0);
        // Forward is +X for identity, so moving forward 1.0 should go to x=1.0
        assert!((pos.0.x - 1.0).abs() < 0.001, "Expected x=1.0, got {}", pos.0.x);
    }
}
