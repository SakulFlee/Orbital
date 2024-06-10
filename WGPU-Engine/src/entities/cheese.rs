use crate::engine::LogicalDevice;

use crate::{
    app::{EntityAction, EntityConfiguration, InputHandler, TEntity, UpdateFrequency},
    engine::{EngineResult, StandardMesh, TMesh},
};

#[derive(Debug, Default)]
pub struct Cheese {
    mesh: Option<StandardMesh>,
}

impl Cheese {
    pub const TAG: &'static str = "Cheese";
}

impl TEntity for Cheese {
    fn entity_configuration(&self) -> EntityConfiguration {
        EntityConfiguration::new(Self::TAG, UpdateFrequency::Slow, true)
    }

    fn update(&mut self, delta_time: f64, _input_handler: &InputHandler) -> Vec<EntityAction> {
        log::debug!("I am a cheese! d: {delta_time}ms");

        vec![]
    }

    fn prepare_render(&mut self, _logical_device: &LogicalDevice) -> EngineResult<()> {
        todo!()

        // let material = StandardMaterial::from_path(logical_device, "cheese.jpg")?;

        // let instances: Vec<StandardInstance> = (-100..=100)
        //     .flat_map(|x| {
        //         (-100..=100).map(move |z| {
        //             StandardInstance::new(
        //                 Vector3::new(x as f32 * 2.5, -1.0, z as f32 * 2.5),
        //                 Quaternion::new(0.0, 0.0, 0.0, 0.0),
        //             )
        //         })
        //     })
        //     .collect();

        // let mesh = ResourceManager::gltf_instanced_mesh_from_path(
        //     logical_device,
        //     "cheese.gltf",
        //     instances,
        //     MaterialLoading::Replace(material),
        // )?;

        // self.mesh = Some(mesh);

        // Ok(())
    }

    fn meshes(&self) -> Vec<&dyn TMesh> {
        vec![self.mesh.as_ref().unwrap()]
    }
}
