use crate::engine::LogicalDevice;
use winit::event::VirtualKeyCode;

use crate::{
    app::{EntityAction, EntityConfiguration, InputHandler, TEntity, UpdateFrequency},
    engine::{EngineResult, StandardMesh, TMesh, VertexPoint},
    entities::{EmptyEntity, OneShotEntity},
};

#[derive(Debug, Default)]
pub struct Square {
    mesh: Option<StandardMesh>,
}

impl Square {
    pub const TAG: &str = "Square";

    /// A square looks like this:
    /// +---+
    /// |   |
    /// +---+
    ///
    /// For rendering optimization we are using Triangles over Quads, thus
    /// we split the square into two triangles:
    /// +---+    +---+
    /// | \ | OR | / |
    /// +---+    +---+
    ///
    /// To draw a single Triangle we need **three** vertex points:
    /// A +
    ///   | \
    /// B +---+ C
    /// > Remember counter-clockwise ordering
    ///
    /// So for two triangles we would need six:
    /// A/F +---+ E
    ///     | \ |
    ///   B +---+ C/D
    ///
    /// However, that's inefficient.
    /// Two points will have the exact same coordinate, thus we can simplify:
    /// A +---+ D
    ///   | \ |
    /// B +---+ C
    ///
    /// Now, we can tell the GPU to render by
    /// an index defines by the index (indices) buffer (see below).
    /// Something like:
    /// 1st Triangle: A -> B -> C
    /// 2nd Triangle: A -> C -> D
    ///
    /// Assuming that the middle of the square is the center of our screen,
    /// we can assign coordinates:
    /// A (-0.5, +0.5) +---+ D (+0.5, +0.5)
    ///                | \ |
    /// B (-0.5, -0.5) +---+ C (+0.5, -0.5)
    /// > For simplicity this is a 2D-view only, totally ignoring the
    /// depth axis (Z).
    /// > Coordinates are in (X, Y) where -X is <- and +X is ->, and,
    /// > +Y is /\ and -Y is \/
    const VERTICES: &[VertexPoint] = &[
        // A
        VertexPoint {
            position_coordinates: [-0.5, 0.5, 0.0],
            texture_coordinates: [0.0, 0.0],
            normal_coordinates: [0.0, 0.0, 0.0],
            tangent: [0.0; 3],
            bitangent: [0.0; 3],
        },
        // B
        VertexPoint {
            position_coordinates: [-0.5, -0.5, 0.0],
            texture_coordinates: [0.0, 1.0],
            normal_coordinates: [0.0, 0.0, 0.0],
            tangent: [0.0; 3],
            bitangent: [0.0; 3],
        },
        // C
        VertexPoint {
            position_coordinates: [0.5, -0.5, 0.0],
            texture_coordinates: [1.0, 1.0],
            normal_coordinates: [0.0, 0.0, 0.0],
            tangent: [0.0; 3],
            bitangent: [0.0; 3],
        },
        // D
        VertexPoint {
            position_coordinates: [0.5, 0.5, 0.0],
            texture_coordinates: [1.0, 0.0],
            normal_coordinates: [0.0, 0.0, 0.0],
            tangent: [0.0; 3],
            bitangent: [0.0; 3],
        },
    ];

    /// Read the [`VERTICES`] comments first!
    ///
    /// Like above, we define the triangles here by referencing the position
    /// inside the above array.
    ///
    /// In our example above we used this model:
    /// A +---+ D
    ///   | \ |
    /// B +---+ C
    ///
    /// And said that the order should be:
    /// 1st Triangle: A -> B -> C
    /// 2nd Triangle: A -> C -> D
    ///
    /// Meaning, the 1st Triangle will be:
    /// A +
    ///   | \
    /// B +---+ C
    ///
    /// And the 2nd will be:
    /// A +---+ D
    ///     \ |
    ///       + C
    ///
    /// Now, the only difference is that we are using index numbers (u32)
    /// instead of letters (A, B, C, D).
    /// An index number starts with zero for the first entry, thus:
    /// A -> 0
    /// B -> 1
    /// C -> 2
    /// D -> 3
    ///
    /// Thus, our draw calls are:
    /// 1st.: 0 -> 1 -> 2
    /// 2nd.: 0 -> 2 -> 3
    ///
    /// The GPU is in "Triangle mode", meaning it expects three numbers
    /// in a row to belong together.
    /// Thus we can just write `0, 1, 2, 0, 2, 3`
    ///
    /// However, to make it easier to read we tell `rustfmt` to skip formatting,
    /// so that we can place 3 numbers in one row and have two rows. :)
    #[rustfmt::skip]
    const INDICES: &[u32] = &[
        0, 1, 2,
        0, 2, 3,
    ];
}
impl TEntity for Square {
    fn entity_configuration(&self) -> EntityConfiguration {
        EntityConfiguration::new(Self::TAG, UpdateFrequency::Slow, true)
    }

    fn update(&mut self, delta_time: f64, input_handler: &InputHandler) -> Vec<EntityAction> {
        log::debug!("Tick! d: {delta_time}ms");

        if input_handler
            .keyboard_input_handler()
            .is_pressed(&VirtualKeyCode::Space)
        {
            // Note: [`UpdateFrequency::Slow`] means we have to hold down Space
            log::debug!("SPACE! We are going to SPACEEEEEEEE!");

            return vec![EntityAction::Spawn(vec![
                Box::new(EmptyEntity::new("empty")),
                Box::new(OneShotEntity::new("one-shot")),
            ])];
        }

        vec![]
    }

    fn prepare_render(&mut self, logical_device: &LogicalDevice) -> EngineResult<()> {
        let mesh = StandardMesh::from_raw_single(
            Some(Self::TAG),
            logical_device,
            Self::VERTICES.into(),
            Self::INDICES.into(),
            None,
        )?;

        self.mesh = Some(mesh);
        Ok(())
    }

    fn meshes(&self) -> Vec<&dyn TMesh> {
        vec![self.mesh.as_ref().unwrap()]
    }
}
