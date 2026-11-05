//! Billboard ECS component and CPU-side description.

use std::sync::Arc;

use cgmath::{Point3, Vector2};
use orbital_ecs_bridge::{EditableField, EditableFields, EditableValue};
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

impl EditableValue for BillboardMode {
    fn to_edit_string(&self) -> String {
        match self {
            Self::Full => "full".to_string(),
            Self::YAxis => "yaxis".to_string(),
            Self::Fixed => "fixed".to_string(),
        }
    }

    fn set_from_str(&mut self, input: &str) -> Result<(), String> {
        *self = match input.trim().to_ascii_lowercase().as_str() {
            "full" => Self::Full,
            "yaxis" | "y-axis" | "y_axis" => Self::YAxis,
            "fixed" => Self::Fixed,
            _ => return Err("expected full, yaxis or fixed".to_string()),
        };
        Ok(())
    }
}

impl EditableFields for Billboard {
    fn editable_fields(&self) -> Vec<EditableField> {
        vec![
            EditableField {
                id: "position".to_string(),
                label: "position".to_string(),
                value: self.position.to_edit_string(),
            },
            EditableField {
                id: "size".to_string(),
                label: "size".to_string(),
                value: self.size.to_edit_string(),
            },
            EditableField {
                id: "rotation".to_string(),
                label: "rotation (rad)".to_string(),
                value: self.rotation.to_edit_string(),
            },
            EditableField {
                id: "pivot".to_string(),
                label: "pivot".to_string(),
                value: self.pivot.to_edit_string(),
            },
            EditableField {
                id: "mode".to_string(),
                label: "mode".to_string(),
                value: self.mode.to_edit_string(),
            },
        ]
    }

    fn set_editable_field(&mut self, id: &str, input: &str) -> Result<(), String> {
        match id {
            "position" => self.position.set_from_str(input),
            "size" => self.size.set_from_str(input),
            "rotation" => self.rotation.set_from_str(input),
            "pivot" => self.pivot.set_from_str(input),
            "mode" => self.mode.set_from_str(input),
            _ => Err(format!("unknown billboard field `{id}`")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn billboard_transform_fields_are_editable() {
        let mut billboard = Billboard::default();

        billboard
            .set_editable_field("position", "1 2 3")
            .expect("position edit failed");
        billboard
            .set_editable_field("size", "4 5")
            .expect("size edit failed");
        billboard
            .set_editable_field("mode", "yaxis")
            .expect("mode edit failed");

        assert_eq!(billboard.position, Point3::new(1.0, 2.0, 3.0));
        assert_eq!(billboard.size, Vector2::new(4.0, 5.0));
        assert_eq!(billboard.mode, BillboardMode::YAxis);
        assert!(billboard.set_editable_field("mode", "sideways").is_err());
    }
}
