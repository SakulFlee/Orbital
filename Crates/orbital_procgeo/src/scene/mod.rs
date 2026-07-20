mod material;
pub use material::*;

mod format;
#[allow(unused_imports)]
pub use format::*;

use std::collections::HashMap;
use std::sync::Arc;

use cgmath::{Quaternion, Vector3};
use orbital_resources::{MaterialShaderDescriptor, MeshDescriptor, Transform};

use crate::shapes;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntityDescriptor {
    pub label: Option<String>,
    pub shape: SceneShape,
    pub material: String,
    pub transform: TransformDef,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransformDef {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl Default for TransformDef {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            rotation: [1.0, 0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}

impl From<TransformDef> for Transform {
    fn from(t: TransformDef) -> Self {
        Transform {
            position: Vector3::new(t.position[0], t.position[1], t.position[2]),
            rotation: Quaternion::new(t.rotation[0], t.rotation[1], t.rotation[2], t.rotation[3]),
            scale: Vector3::new(t.scale[0], t.scale[1], t.scale[2]),
        }
    }
}

impl From<Transform> for TransformDef {
    fn from(t: Transform) -> Self {
        Self {
            position: [t.position.x, t.position.y, t.position.z],
            rotation: [t.rotation.s, t.rotation.v.x, t.rotation.v.y, t.rotation.v.z],
            scale: [t.scale.x, t.scale.y, t.scale.z],
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SceneShape {
    Plane { size: [f32; 2], subdivisions: u32 },
    Box { size: [f32; 3] },
    UvSphere { radius: f32, segments: u32, rings: u32 },
    Cylinder { radius: f32, height: f32, segments: u32 },
    Cone { radius: f32, height: f32, segments: u32 },
    Torus { major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32 },
    Capsule { radius: f32, height: f32, segments: u32, rings: u32 },
    Disk { radius: f32, segments: u32 },
    Grid { width: f32, depth: f32, cols: u32, rows: u32 },
}

impl SceneShape {
    pub fn generate(&self) -> MeshDescriptor {
        match *self {
            SceneShape::Plane { size, subdivisions } => {
                shapes::plane(cgmath::Vector2::new(size[0], size[1]), subdivisions)
            }
            SceneShape::Box { size } => {
                shapes::box_(cgmath::Vector3::new(size[0], size[1], size[2]))
            }
            SceneShape::UvSphere { radius, segments, rings } => {
                shapes::uv_sphere(radius, segments, rings)
            }
            SceneShape::Cylinder { radius, height, segments } => {
                shapes::cylinder(radius, height, segments)
            }
            SceneShape::Cone { radius, height, segments } => {
                shapes::cone(radius, height, segments)
            }
            SceneShape::Torus { major_radius, minor_radius, major_segments, minor_segments } => {
                shapes::torus(major_radius, minor_radius, major_segments, minor_segments)
            }
            SceneShape::Capsule { radius, height, segments, rings } => {
                shapes::capsule(radius, height, segments, rings)
            }
            SceneShape::Disk { radius, segments } => {
                shapes::disk(radius, segments)
            }
            SceneShape::Grid { width, depth, cols, rows } => {
                shapes::grid(width, depth, cols, rows)
            }
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SceneBuilder {
    pub entities: Vec<EntityDescriptor>,
    pub materials: HashMap<String, SceneMaterial>,
}

impl SceneBuilder {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            materials: HashMap::new(),
        }
    }

    pub fn add_entity(&mut self, entity: EntityDescriptor) -> &mut Self {
        self.entities.push(entity);
        self
    }

    pub fn add_material(&mut self, name: &str, material: SceneMaterial) -> &mut Self {
        self.materials.insert(name.to_string(), material);
        self
    }

    pub fn build(
        &self,
    ) -> Vec<(MeshDescriptor, Arc<MaterialShaderDescriptor>, Transform)> {
        let mut result = Vec::new();

        for entity in &self.entities {
            let mesh = entity.shape.generate();

            let material = self
                .materials
                .get(&entity.material)
                .cloned()
                .unwrap_or(SceneMaterial::Color {
                    albedo: [0.5, 0.5, 0.5, 1.0],
                    metallic: 0.0,
                    roughness: 0.5,
                });

            let mat_desc = material.into_material_shader();
            let transform = Transform::from(entity.transform.clone());

            result.push((mesh, mat_desc, transform));
        }

        result
    }

    pub fn to_ron(&self) -> Result<String, ron::Error> {
        ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
    }

    pub fn from_ron(input: &str) -> Result<Self, ron::error::SpannedError> {
        ron::from_str(input)
    }

    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let s = self.to_ron()?;
        std::fs::write(path, s)?;
        Ok(())
    }

    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let s = std::fs::read_to_string(path)?;
        let scene = Self::from_ron(&s)?;
        Ok(scene)
    }
}

impl Default for SceneBuilder {
    fn default() -> Self {
        Self::new()
    }
}
