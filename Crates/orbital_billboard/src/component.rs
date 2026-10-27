//! Billboard ECS component and CPU-side description.

use std::sync::Arc;

use cgmath::{Point3, Vector2};
use orbital_material_shader::{AlphaMode, MaterialShaderDescriptor};
use orbital_mesh::MeshDescriptor;
use orbital_texture::TextureDescriptor;

use crate::material::{lit_material, unlit_material};

/// How a billboard is oriented relative to the camera.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BillboardMode {
    /// Face the camera in both axes.
    #[default]
    Full,
    /// Rotate around the world Y axis; remains upright.
    YAxis,
    /// Keep the quad in its fixed local XY plane.
    Fixed,
}

/// A camera-facing textured quad.
///
/// The component is cheap to clone: the material and optional mesh are shared.
/// [`BillboardModule`](crate::BillboardModule) turns it into a normal
/// [`ModelDescriptor`](orbital_ecs_bridge::ModelDescriptor), so billboards use
/// the existing opaque/deferred, transparent, depth-test and sorting paths.
#[derive(Debug, Clone)]
pub struct Billboard {
    /// World-space center before pivot adjustment.
    pub position: Point3<f32>,
    /// Width and height in world units.
    pub size: Vector2<f32>,
    /// Rotation around the billboard normal, in radians.
    pub rotation: f32,
    /// Normalized pivot in `[0, 1]`; `(0.5, 0.5)` is centered.
    pub pivot: Vector2<f32>,
    pub mode: BillboardMode,
    /// Material used for the quad. Use the constructors or `with_material`.
    pub material: Arc<MaterialShaderDescriptor>,
    /// Optional custom mesh. `None` uses the shared unit quad.
    pub mesh: Option<Arc<MeshDescriptor>>,
}

impl Billboard {
    pub fn unlit(size: Vector2<f32>, alpha_mode: AlphaMode) -> Self {
        Self::from_material(
            size,
            unlit_material(
                TextureDescriptor::uniform_rgba_white(false),
                [1.0, 1.0, 1.0, 1.0],
                alpha_mode,
            ),
        )
    }

    pub fn unlit_textured(
        size: Vector2<f32>,
        texture: TextureDescriptor,
        color: [f32; 4],
        alpha_mode: AlphaMode,
    ) -> Self {
        Self::from_material(size, unlit_material(texture, color, alpha_mode))
    }

    pub fn lit(size: Vector2<f32>, alpha_mode: AlphaMode) -> Self {
        Self::from_material(
            size,
            lit_material(
                TextureDescriptor::uniform_rgba_white(true),
                [1.0, 1.0, 1.0, 1.0],
                alpha_mode,
            ),
        )
    }

    pub fn lit_textured(
        size: Vector2<f32>,
        texture: TextureDescriptor,
        color: [f32; 4],
        alpha_mode: AlphaMode,
    ) -> Self {
        Self::from_material(size, lit_material(texture, color, alpha_mode))
    }

    pub fn from_material(size: Vector2<f32>, material: Arc<MaterialShaderDescriptor>) -> Self {
        Self {
            position: Point3::new(0.0, 0.0, 0.0),
            size,
            rotation: 0.0,
            pivot: Vector2::new(0.5, 0.5),
            mode: BillboardMode::Full,
            material,
            mesh: None,
        }
    }

    pub fn with_position(mut self, position: Point3<f32>) -> Self {
        self.position = position;
        self
    }

    pub fn with_size(mut self, size: Vector2<f32>) -> Self {
        self.size = size;
        self
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_pivot(mut self, pivot: Vector2<f32>) -> Self {
        self.pivot = pivot;
        self
    }

    pub fn with_mode(mut self, mode: BillboardMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_material(mut self, material: Arc<MaterialShaderDescriptor>) -> Self {
        self.material = material;
        self
    }

    pub fn with_mesh(mut self, mesh: Arc<MeshDescriptor>) -> Self {
        self.mesh = Some(mesh);
        self
    }
}

impl Default for Billboard {
    fn default() -> Self {
        Self::unlit(Vector2::new(1.0, 1.0), AlphaMode::Opaque)
    }
}
