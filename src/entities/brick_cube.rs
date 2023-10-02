use cgmath::{Deg, InnerSpace, Quaternion, Rotation3, Vector3};

use crate::engine::{
    LogicalDevice, MaterialLoading, ResourceManager, StandardInstance, StandardMaterial, TInstance,
};

use crate::{
    app::{EntityAction, EntityConfiguration, InputHandler, TEntity, UpdateFrequency},
    engine::{EngineResult, StandardMesh, TMesh},
};

#[derive(Debug, Default)]
pub struct BrickCube {
    mesh: Option<StandardMesh>,
}

impl BrickCube {
    pub const TAG: &str = "BrickCube";
}

impl TEntity for BrickCube {
    fn entity_configuration(&self) -> EntityConfiguration {
        EntityConfiguration::new(Self::TAG, UpdateFrequency::Slow, true)
    }

    fn update(&mut self, delta_time: f64, _input_handler: &InputHandler) -> Vec<EntityAction> {
        log::debug!("I am a cheese! d: {delta_time}ms");

        vec![]
    }

    fn prepare_render(&mut self, logical_device: &LogicalDevice) -> EngineResult<()> {
        let material = StandardMaterial::from_path(
            logical_device,
            "cube/cube-diffuse.jpg",
            "cube/cube-normal.png",
        )?;

        let instances: Vec<StandardInstance> = (-100..=100)
            .flat_map(|x| {
                (-100..=100).map(move |z| {
                    let position = Vector3::new(x as f32 * 1.0, -1.0, z as f32 * 1.0);
                    StandardInstance::new(
                        position,
                        // Quaternion::from_axis_angle(position.normalize(), Deg(45.0)),
                        Quaternion::new(0.0, 0.0, 0.0, 0.0)
                    )
                })
            })
            .collect();

        let mesh = ResourceManager::gltf_instanced_mesh_from_path(
            logical_device,
            "cube/cube.gltf",
            instances,
            MaterialLoading::Replace(material),
        )?;

        self.mesh = Some(mesh);

        Ok(())
    }

    fn meshes(&self) -> Vec<&dyn TMesh> {
        vec![self.mesh.as_ref().unwrap()]
    }
}
