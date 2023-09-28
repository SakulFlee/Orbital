use crate::engine::{LogicalDevice, ResourceManager, StandardMaterial};

use crate::{
    app::{EntityAction, EntityConfiguration, InputHandler, TEntity, UpdateFrequency},
    engine::{EngineResult, StandardMesh, TMesh},
};

#[derive(Debug, Default)]
pub struct Cube {
    mesh: Option<StandardMesh>,
}

impl Cube {
    pub const TAG: &str = "Cube";
}

impl TEntity for Cube {
    fn entity_configuration(&self) -> EntityConfiguration {
        EntityConfiguration::new(Self::TAG, UpdateFrequency::Slow, true)
    }

    fn update(&mut self, delta_time: f64, _input_handler: &InputHandler) -> Vec<EntityAction> {
        log::debug!("I am a cube! d: {delta_time}ms");

        vec![]
    }

    fn prepare_render(&mut self, logical_device: &LogicalDevice) -> EngineResult<()> {
        let material = Box::new(StandardMaterial::from_path(logical_device, "cheese.jpg")?);

        let mut mesh = ResourceManager::gltf_mesh_from_path(logical_device, "cheese.glb")?;

        mesh.set_material(material);

        self.mesh = Some(mesh);

        Ok(())
    }

    fn meshes(&self) -> Vec<&dyn TMesh> {
        vec![self.mesh.as_ref().unwrap()]
    }
}
