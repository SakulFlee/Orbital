use wgpu::{Device, Queue};
use winit::event::VirtualKeyCode;

use crate::{
    app::{EntityAction, EntityConfiguration, InputHandler, TEntity, UpdateFrequency},
    engine::{StandardMesh, TMesh, VertexPoint},
    entities::{EmptyEntity, OneShotEntity},
};

#[derive(Debug, Default)]
pub struct Cube {
    mesh: Option<StandardMesh>,
}

impl Cube {
    pub const TAG: &str = "Cube";

    const VERTICES: &[VertexPoint] = &[
        // A
        VertexPoint {
            position_coordinates: [0.0, 0.35, 0.0],
            texture_coordinates: [1.0, 1.0],
            normal_coordinates: [0.0, 0.0, 0.0],
        },
        // B
        VertexPoint {
            position_coordinates: [-0.35, -0.35, 0.0],
            texture_coordinates: [-1.0, 0.0],
            normal_coordinates: [0.0, 0.0, 0.0],
        },
        // C
        VertexPoint {
            position_coordinates: [0.35, -0.35, 0.0],
            texture_coordinates: [0.0, -1.0],
            normal_coordinates: [0.0, 0.0, 0.0],
        },
    ];

    const INDICES: &[u32] = &[0, 1, 2];
}
impl TEntity for Cube {
    fn get_entity_configuration(&self) -> EntityConfiguration {
        EntityConfiguration::new(Self::TAG, UpdateFrequency::Slow, true)
    }

    fn update(&mut self, delta_time: f64, input_handler: &InputHandler) -> EntityAction {
        log::debug!("Tick! d: {delta_time}ms");

        if input_handler.is_key_pressed(&VirtualKeyCode::Space) {
            // Note: [`UpdateFrequency::Slow`] means we have to hold down Space
            log::debug!("SPACE! We are going to SPACEEEEEEEE!");

            return EntityAction::Spawn(vec![
                Box::new(EmptyEntity::new("empty")),
                Box::new(OneShotEntity::new("one-shot")),
            ]);
        }

        EntityAction::Keep
    }

    fn prepare_render(&mut self, device: &Device, _queue: &Queue) {
        let mesh = StandardMesh::from_raw(
            Some(&Self::TAG),
            device,
            Self::VERTICES.into(),
            Self::INDICES.into(),
            0..1,
            None,
        );

        self.mesh = Some(mesh);
    }

    fn get_meshes(&self) -> Vec<&dyn TMesh> {
        vec![self.mesh.as_ref().unwrap()]
    }
}
