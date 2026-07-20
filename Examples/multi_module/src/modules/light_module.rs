use orbital::app::Module;
use orbital::cgmath::{Point3, Vector3};
use orbital::ecs::{System, World};
use orbital::ecs_bridge::{LightDescriptorEcs, LightDirty, Position};
use orbital::resources::ShadowCaster;

pub struct LightModule;

impl Module for LightModule {
    fn setup(
        &self,
        ecs: &mut World,
        _device: &orbital::wgpu::Device,
        _queue: &orbital::wgpu::Queue,
    ) -> Vec<Box<dyn System>> {
        // Spawn a directional light
        let light = ecs.spawn_entity();
        ecs.attach_component(
            &light,
            LightDescriptorEcs::new_directional(
                Vector3::new(-1.0, -1.0, -1.0),
                Vector3::new(1.0, 1.0, 1.0),
                1.0,
            ),
        )
        .unwrap();
        ecs.attach_component(&light, Position(Point3::new(0.0, 0.0, 0.0)))
            .unwrap();
        ecs.attach_component(&light, LightDirty(true)).unwrap();
        ecs.attach_component(&light, ShadowCaster::default()).unwrap();

        // Spawn a point light
        let light2 = ecs.spawn_entity();
        ecs.attach_component(
            &light2,
            LightDescriptorEcs::new_point(Vector3::new(1.0, 1.0, 1.0), 5.0),
        )
        .unwrap();
        ecs.attach_component(&light2, Position(Point3::new(3.0, 3.0, 3.0)))
            .unwrap();
        ecs.attach_component(&light2, LightDirty(true)).unwrap();

        vec![] // Light has no per-frame systems
    }
}
