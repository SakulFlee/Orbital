use cgmath::{Quaternion, Vector2, Vector3, Zero};
use easy_gltf::Model;
use logical_device::LogicalDevice;

use crate::engine::{
    logical_device, DiffuseTexture, EngineError, EngineResult, MaterialLoading, NormalTexture,
    StandardInstance, StandardMaterial, StandardMesh, TInstance, TMaterial, VertexPoint,
};

pub trait ToStandardMesh {
    fn to_mesh(
        &self,
        logical_device: &LogicalDevice,
        material_loading: MaterialLoading,
    ) -> EngineResult<StandardMesh> {
        self.to_instanced_mesh(
            logical_device,
            material_loading,
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
        material_loading: MaterialLoading,
        instances: Vec<StandardInstance>,
    ) -> EngineResult<StandardMesh>;
}

impl ToStandardMesh for Model {
    fn to_instanced_mesh(
        &self,
        logical_device: &LogicalDevice,
        material_loading: MaterialLoading,
        instances: Vec<StandardInstance>,
    ) -> EngineResult<StandardMesh> {
        let mut vertices: Vec<VertexPoint> = self.vertices().iter().map(|x| x.into()).collect();

        let mut indices = self
            .indices()
            .map(|x| Ok(x.to_vec()))
            .unwrap_or(Err(EngineError::GltfNoIndices))?;

        let mut position_counter = vec![0; vertices.len()];

        // Calculate tangent & bitangent
        for index in indices.chunks_mut(3) {
            let vertex_0 = vertices[index[0] as usize];
            let vertex_1 = vertices[index[1] as usize];
            let vertex_2 = vertices[index[2] as usize];

            let position_0: Vector3<_> = vertex_0.position_coordinates.into();
            let position_1: Vector3<_> = vertex_1.position_coordinates.into();
            let position_2: Vector3<_> = vertex_2.position_coordinates.into();

            let uv_0: Vector2<_> = vertex_0.texture_coordinates.into();
            let uv_1: Vector2<_> = vertex_1.texture_coordinates.into();
            let uv_2: Vector2<_> = vertex_2.texture_coordinates.into();

            // Calculate edges of triangle
            let delta_pos_1 = position_1 - position_0;
            let delta_pos_2 = position_2 - position_0;

            // Calculate edge of triangle
            let delta_uv_1 = uv_1 - uv_0;
            let delta_uv_2 = uv_2 - uv_0;

            // Calculate tangent and bitangent
            // Based on:
            //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
            //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
            let r = 1.0 / (delta_uv_1.x * delta_uv_2.y - delta_uv_1.y * delta_uv_2.x);
            let tangent = (delta_pos_1 * delta_uv_2.y - delta_pos_2 * delta_uv_1.y) * r;
            // Note:: Bitangent is flipped to enable right-handed normal
            // maps with wgpu texture coordinate system
            let bitangent = (delta_pos_2 * delta_uv_1.x - delta_pos_1 * delta_uv_2.x) * -r;

            // Set tangent & bitangent
            vertices[index[0] as usize].tangent = tangent.into();
            vertices[index[1] as usize].tangent = tangent.into();
            vertices[index[2] as usize].tangent = tangent.into();
            vertices[index[0] as usize].bitangent = bitangent.into();
            vertices[index[1] as usize].bitangent = bitangent.into();
            vertices[index[2] as usize].bitangent = bitangent.into();

            // Used below
            position_counter[index[0] as usize] += 1;
            position_counter[index[1] as usize] += 1;
            position_counter[index[2] as usize] += 1;
        }

        // Average tangents/bitangents
        for (i, n) in position_counter.into_iter().enumerate() {
            let denom = 1.0 / n as f32;

            let v = &mut vertices[i];
            v.tangent = (Vector3::from(v.tangent) * denom).into();
            v.bitangent = (Vector3::from(v.bitangent) * denom).into();
        }

        let material: Option<Box<dyn TMaterial>> = match material_loading {
            MaterialLoading::Ignore => None,
            MaterialLoading::Try => match &self.material().pbr.base_color_texture {
                Some(base_color_texture) => {
                    match DiffuseTexture::from_bytes(logical_device, base_color_texture, None) {
                        Ok(diffuse_texture) => match NormalTexture::empty(logical_device) {
                            Ok(normal_texture) => {
                                match StandardMaterial::from_texture(
                                    logical_device,
                                    diffuse_texture,
                                    normal_texture,
                                ) {
                                    Ok(material) => Some(Box::new(material)),
                                    Err(_) => None,
                                }
                            }
                            Err(_) => None,
                        },
                        Err(_) => None,
                    }
                }
                None => None,
            },
            MaterialLoading::Replace(material) => Some(Box::new(material)),
        };

        StandardMesh::from_raw(None, logical_device, vertices, indices, instances, material)
    }
}
