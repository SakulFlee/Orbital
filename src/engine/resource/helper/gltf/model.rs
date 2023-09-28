use cgmath::{Quaternion, Vector3, Zero};
use easy_gltf::Model;
use logical_device::LogicalDevice;

use crate::engine::{
    logical_device, DiffuseTexture, EngineError, EngineResult, StandardInstance, StandardMaterial,
    StandardMesh, TInstance, TMaterial, VertexPoint,
};

pub trait ToStandardMesh {
    fn to_mesh(&self, logical_device: &LogicalDevice) -> EngineResult<StandardMesh> {
        self.to_instanced_mesh(
            logical_device,
            vec![StandardInstance::new(
                Vector3::zero(),
                Quaternion {
                    v: Vector3::zero(),
                    s: 0.0,
                },
            )],
        )
    }

    fn to_instanced_mesh(
        &self,
        logical_device: &LogicalDevice,
        instances: Vec<StandardInstance>,
    ) -> EngineResult<StandardMesh>;
}

impl ToStandardMesh for Model {
    fn to_instanced_mesh(
        &self,
        logical_device: &LogicalDevice,
        instances: Vec<StandardInstance>,
    ) -> EngineResult<StandardMesh> {
        let vertices: Vec<VertexPoint> = self.vertices().iter().map(|x| x.into()).collect();

        let indices = self
            .indices()
            .map(|x| Ok(x.iter().cloned().collect()))
            .unwrap_or(Err(EngineError::GltfNoIndices))?;

        let material: Option<Box<dyn TMaterial>> = match &self.material().pbr.base_color_texture {
            Some(base_color_texture) => {
                match DiffuseTexture::from_bytes(logical_device, &base_color_texture, None) {
                    Ok(diffuse_texture) => {
                        match StandardMaterial::from_texture(logical_device, diffuse_texture) {
                            Ok(material) => Some(Box::new(material)),
                            Err(_) => None,
                        }
                    }
                    Err(_) => None,
                }
            }
            None => None,
        };

        Ok(StandardMesh::from_raw(
            None,
            logical_device,
            vertices,
            indices,
            instances,
            material,
        )?)
    }
}
