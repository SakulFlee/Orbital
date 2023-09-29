use std::path::Path;

use cgmath::{Quaternion, Vector3, Zero};
use wgpu::{Buffer, BufferUsages};

use crate::engine::{
    BufferHelper, EngineError, EngineResult, LogicalDevice, ResourceManager, StandardInstance,
    StandardMaterial, TInstance, TMaterial, TMesh, ToStandardMesh, VertexPoint,
};

use super::MaterialLoading;

#[derive(Debug)]
pub struct StandardMesh {
    name: Option<String>,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: u32,
    instances: Vec<StandardInstance>,
    instance_buffer: Buffer,
    material: Box<dyn TMaterial>,
}

impl StandardMesh {
    pub const MISSING_TEXTURE: &str = "missing_texture.png";

    pub fn from_gltf_single<P>(
        logical_device: &LogicalDevice,
        path: P,
        material_loading: MaterialLoading,
    ) -> EngineResult<Self>
    where
        P: AsRef<Path>,
    {
        Self::from_gltf_instanced(
            logical_device,
            path,
            vec![StandardInstance::new(
                Vector3::zero(),
                Quaternion {
                    v: Vector3::zero(),
                    s: 0.0,
                },
            )],
            material_loading,
        )
    }

    pub fn from_gltf_instanced<P>(
        logical_device: &LogicalDevice,
        path: P,
        instances: Vec<StandardInstance>,
        material_loading: MaterialLoading,
    ) -> EngineResult<Self>
    where
        P: AsRef<Path>,
    {
        let scenes = ResourceManager::read_resource_gltf(&path)?;

        if scenes.len() == 0 {
            return Err(EngineError::GltfNoScene);
        }
        if scenes.len() > 1 {
            log::warn!(
                "GLTF '{:?}' has multiple scenes. Only the first one will be used!",
                path.as_ref()
            );
        }

        let scene = scenes.first().unwrap();

        if scene.models.len() == 0 {
            return Err(EngineError::GltfNoModel);
        }
        if scene.models.len() > 1 {
            log::warn!(
                "GLTF '{:?}' has multiple models. Only the first one will be used!",
                path.as_ref()
            );
        }

        Ok(scene.models.first().unwrap().to_instanced_mesh(
            logical_device,
            material_loading,
            instances,
        )?)
    }

    pub fn from_raw_single(
        name: Option<&str>,
        logical_device: &LogicalDevice,
        vertices: Vec<VertexPoint>,
        indices: Vec<u32>,
        material: Option<Box<dyn TMaterial>>,
    ) -> EngineResult<Self> {
        Self::from_raw(
            name,
            logical_device,
            vertices,
            indices,
            vec![StandardInstance::new(
                Vector3::zero(),
                Quaternion {
                    v: Vector3::zero(),
                    s: 0.0,
                },
            )],
            material,
        )
    }

    pub fn from_raw(
        name: Option<&str>,
        logical_device: &LogicalDevice,
        vertices: Vec<VertexPoint>,
        indices: Vec<u32>,
        instances: Vec<StandardInstance>,
        material: Option<Box<dyn TMaterial>>,
    ) -> EngineResult<Self> {
        let label = name.unwrap_or("Unknown");

        let vertex_buffer = logical_device.make_buffer(
            Some(&format!("{} Vertex Buffer", label)),
            &vertices,
            BufferUsages::VERTEX,
        );
        let index_buffer = logical_device.make_buffer(
            Some(&format!("{} Index Buffer", label)),
            &indices,
            BufferUsages::INDEX,
        );

        let instance_uniform = instances
            .iter()
            .map(|x| x.to_instance_uniform())
            .collect::<Vec<_>>();
        let instance_buffer = logical_device.make_buffer(
            Some(&format!("{} Instance Buffer", label)),
            &instance_uniform,
            BufferUsages::VERTEX,
        );

        let material = match material {
            Some(material) => material,
            None => Box::new(StandardMaterial::from_path(
                logical_device,
                Self::MISSING_TEXTURE,
            )?),
        };

        Ok(Self {
            name: name.map(|x| x.to_string()),
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            instances,
            instance_buffer,
            material,
        })
    }

    pub fn set_material(&mut self, material: Box<dyn TMaterial>) {
        self.material = material;
    }
}

impl TMesh for StandardMesh {
    fn vertex_buffer(&self) -> &Buffer {
        &self.vertex_buffer
    }

    fn index_buffer(&self) -> &Buffer {
        &self.index_buffer
    }

    fn index_count(&self) -> u32 {
        self.index_count
    }

    fn instances(&mut self) -> &mut Vec<StandardInstance> {
        &mut self.instances
    }

    fn instance_count(&self) -> u32 {
        self.instances.len() as u32
    }

    fn instance_buffer(&self) -> &Buffer {
        &self.instance_buffer
    }

    fn material(&self) -> &dyn TMaterial {
        self.material.as_ref()
    }

    fn name(&self) -> Option<String> {
        self.name.clone()
    }
}

impl ResourceManager {
    pub fn gltf_mesh_from_path<P>(
        logical_device: &LogicalDevice,
        file_path: P,
        material_loading: MaterialLoading,
    ) -> EngineResult<StandardMesh>
    where
        P: AsRef<Path>,
    {
        StandardMesh::from_gltf_single(logical_device, file_path, material_loading)
    }

    pub fn gltf_instanced_mesh_from_path<P>(
        logical_device: &LogicalDevice,
        file_path: P,
        instance: Vec<StandardInstance>,
        material_loading: MaterialLoading,
    ) -> EngineResult<StandardMesh>
    where
        P: AsRef<Path>,
    {
        StandardMesh::from_gltf_instanced(logical_device, file_path, instance, material_loading)
    }
}
