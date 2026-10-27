//! ECS system that turns [`Billboard`] components into normal models.

use std::sync::Arc;

use cgmath::{InnerSpace, Matrix3, Point3, Quaternion, Rad, Rotation3, Vector2, Vector3};
use orbital_app::Module;
use orbital_ecs::{Commands, ComponentAccess, Entity, System, World};
use orbital_ecs_bridge::{
    ActiveCamera, ModelDescriptor, ModelDirty, ModelInstances, Position, Rotation,
};
use orbital_math::Transform;
use orbital_mesh::MeshDescriptor;
use orbital_vertex::Vertex;
use wgpu::{Device, Queue};

use crate::component::{Billboard, BillboardMode};

/// Shared unit quad used by billboards without a custom mesh.
#[derive(Debug)]
pub(crate) struct BillboardQuadMesh(pub Arc<MeshDescriptor>);

/// Adds billboard realization to an app.
///
/// Add this module after the module that spawns billboard components so the
/// realization system has a chance to see them on the first frame.
pub struct BillboardModule;

impl Module for BillboardModule {
    fn setup(&self, ecs: &mut World, _device: &Device, _queue: &Queue) -> Vec<Box<dyn System>> {
        ecs.insert_resource(BillboardQuadMesh(Arc::new(unit_quad())));
        vec![Box::new(BillboardSystem::new())]
    }
}

pub(crate) struct BillboardSystem {
    access: ComponentAccess,
}

impl BillboardSystem {
    fn new() -> Self {
        Self {
            access: ComponentAccess::new()
                .reads::<Billboard>()
                .reads::<ActiveCamera>()
                .reads::<Position>()
                .reads::<Rotation>()
                .writes::<ModelDescriptor>()
                .writes::<ModelInstances>()
                .writes::<ModelDirty>(),
        }
    }
}

impl System for BillboardSystem {
    fn name(&self) -> &str {
        "billboard_realize"
    }

    fn access(&self) -> &ComponentAccess {
        &self.access
    }

    fn run(&mut self, world: &World, commands: &mut Commands) {
        let Some(billboards) = world.get_component_store::<Billboard>() else {
            return;
        };
        let Some(quad) = world.get_resource::<BillboardQuadMesh>() else {
            return;
        };

        let camera = camera_pose(world);
        let mut instances_store = world.get_component_store_mut::<ModelInstances>();
        let mut dirty_store = world.get_component_store_mut::<ModelDirty>();

        for &entity_index in billboards.dense.as_slice() {
            let Some(index) = billboards.sparse.get(entity_index).copied().flatten() else {
                continue;
            };
            let billboard = &billboards.components[index];
            let transform = billboard_transform(billboard, camera);
            let entity = Entity::new(entity_index, world.generation(entity_index));

            let existing = instances_store
                .as_ref()
                .and_then(|store| store.sparse.get(entity_index).copied().flatten());

            if let Some(index) = existing {
                if let Some(store) = instances_store.as_mut() {
                    let instances = &mut store.get_mut_store().components[index];
                    if let Some((_, instance)) = instances.0.iter_mut().next() {
                        *instance = transform;
                    } else {
                        instances.add_instance(transform);
                    }
                }

                if let Some(store) = dirty_store.as_mut() {
                    if let Some(index) = store.sparse.get(entity_index).copied().flatten() {
                        store.get_mut_store().components[index].0 = true;
                    } else {
                        commands.attach_component(&entity, ModelDirty(true));
                    }
                } else {
                    commands.attach_component(&entity, ModelDirty(true));
                }
            } else {
                let mut instances = ModelInstances::new();
                instances.add_instance(transform);
                commands.attach_component(
                    &entity,
                    ModelDescriptor {
                        label: format!("Billboard {entity_index}"),
                        mesh: billboard.mesh.clone().unwrap_or_else(|| quad.0.clone()),
                        materials: vec![billboard.material.clone()],
                    },
                );
                commands.attach_component(&entity, instances);
                commands.attach_component(&entity, ModelDirty(true));
            }
        }
    }
}

fn camera_pose(world: &World) -> Option<(Point3<f32>, Quaternion<f32>)> {
    let active = world.get_resource::<ActiveCamera>()?.0;
    let position = world.get_component_store::<Position>()?;
    let rotation = world.get_component_store::<Rotation>()?;

    let position_index = position.sparse.get(active.index).copied().flatten()?;
    let rotation_index = rotation.sparse.get(active.index).copied().flatten()?;
    Some((
        position.components[position_index].0,
        rotation.components[rotation_index].0,
    ))
}

fn billboard_transform(
    billboard: &Billboard,
    camera: Option<(Point3<f32>, Quaternion<f32>)>,
) -> Transform {
    let rotation = match billboard.mode {
        BillboardMode::Full => {
            let camera_rotation = camera
                .map(|(_, rotation)| rotation)
                .unwrap_or_else(identity);
            let (forward, right, up) = Rotation(camera_rotation).forward_right_up();
            let basis = Matrix3::from_cols(right, up, -forward);
            let billboard_rotation = Quaternion::from(basis);
            billboard_rotation * spin(billboard.rotation)
        }
        BillboardMode::YAxis => {
            let camera_position = camera
                .map(|(position, _)| position)
                .unwrap_or_else(identity_point);
            let offset = camera_position - billboard.position;
            let horizontal = Vector3::new(offset.x, 0.0, offset.z);
            let forward = if horizontal.magnitude2() > 1.0e-8 {
                horizontal.normalize()
            } else {
                Vector3::unit_z()
            };
            let up = Vector3::unit_y();
            let right = up.cross(forward).normalize();
            let basis = Matrix3::from_cols(right, up, forward);
            Quaternion::from(basis) * spin(billboard.rotation)
        }
        BillboardMode::Fixed => spin(billboard.rotation),
    };

    let size = Vector2::new(billboard.size.x.max(0.0001), billboard.size.y.max(0.0001));
    let right = rotation * Vector3::unit_x();
    let up = rotation * Vector3::unit_y();
    let pivot_offset =
        right * ((billboard.pivot.x - 0.5) * size.x) + up * ((billboard.pivot.y - 0.5) * size.y);

    Transform {
        position: Vector3::new(
            billboard.position.x - pivot_offset.x,
            billboard.position.y - pivot_offset.y,
            billboard.position.z - pivot_offset.z,
        ),
        rotation,
        scale: Vector3::new(size.x, size.y, 1.0),
    }
}

fn spin(radians: f32) -> Quaternion<f32> {
    Quaternion::from_axis_angle(Vector3::unit_z(), Rad(radians))
}

fn identity() -> Quaternion<f32> {
    Quaternion::new(1.0, 0.0, 0.0, 0.0)
}

fn identity_point() -> Point3<f32> {
    Point3::new(0.0, 0.0, 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use orbital_material_shader::{AlphaMode, RenderQueue};

    fn approx(a: Vector3<f32>, b: Vector3<f32>) -> bool {
        (a - b).magnitude() < 1.0e-5
    }

    #[test]
    fn quad_has_expected_geometry() {
        let mesh = unit_quad();
        assert_eq!(mesh.vertices.len(), 4);
        assert_eq!(mesh.indices, vec![0, 1, 2, 0, 2, 3]);
        assert_eq!(mesh.vertices[0].uv, Vector2::new(0.0, 1.0));
        assert_eq!(mesh.vertices[2].uv, Vector2::new(1.0, 0.0));
    }

    #[test]
    fn fixed_billboard_applies_pivot_and_scale() {
        let billboard = Billboard::default()
            .with_size(Vector2::new(2.0, 2.0))
            .with_pivot(Vector2::new(0.0, 0.0))
            .with_mode(BillboardMode::Fixed);

        let transform = billboard_transform(&billboard, None);

        assert!(approx(transform.scale, Vector3::new(2.0, 2.0, 1.0)));
        assert!(approx(transform.position, Vector3::new(1.0, 1.0, 0.0)));
    }

    #[test]
    fn y_axis_billboard_keeps_world_up() {
        let billboard = Billboard::default()
            .with_position(Point3::new(0.0, 0.0, 0.0))
            .with_mode(BillboardMode::YAxis);
        let camera = Some((
            Point3::new(0.0, 0.0, 5.0),
            Quaternion::from_axis_angle(Vector3::unit_y(), Rad(0.0)),
        ));

        let transform = billboard_transform(&billboard, camera);

        assert!(approx(
            transform.rotation * Vector3::unit_y(),
            Vector3::unit_y()
        ));
        assert!(approx(
            transform.rotation * Vector3::unit_z(),
            Vector3::unit_z()
        ));
    }

    #[test]
    fn full_billboard_faces_the_camera() {
        let billboard = Billboard::default().with_mode(BillboardMode::Full);
        let camera_rotation =
            Quaternion::from_axis_angle(Vector3::unit_y(), Rad(std::f32::consts::PI));
        let camera = Some((Point3::new(5.0, 0.0, 0.0), camera_rotation));

        let transform = billboard_transform(&billboard, camera);

        assert!(approx(
            transform.rotation * Vector3::unit_z(),
            Vector3::unit_x()
        ));
    }

    #[test]
    fn blend_billboards_use_the_transparent_queue() {
        let unlit = Billboard::unlit(Vector2::new(1.0, 1.0), AlphaMode::Blend);
        assert_eq!(unlit.material.render_queue, RenderQueue::Transparent);
        assert!(!unlit.material.depth_write);
        assert!(unlit.material.entrypoint_fragment_gbuffer.is_none());

        let lit = Billboard::lit(Vector2::new(1.0, 1.0), AlphaMode::Opaque);
        assert_eq!(lit.material.render_queue, RenderQueue::Opaque);
        assert!(lit.material.entrypoint_fragment_gbuffer.is_some());
    }
}

fn unit_quad() -> MeshDescriptor {
    let normal = Vector3::unit_z();
    let tangent = Vector3::unit_x();
    let bitangent = Vector3::unit_y();

    let vertices = vec![
        Vertex::new_with_bitangent(
            Vector3::new(-0.5, -0.5, 0.0),
            normal,
            tangent,
            bitangent,
            Vector2::new(0.0, 1.0),
        ),
        Vertex::new_with_bitangent(
            Vector3::new(0.5, -0.5, 0.0),
            normal,
            tangent,
            bitangent,
            Vector2::new(1.0, 1.0),
        ),
        Vertex::new_with_bitangent(
            Vector3::new(0.5, 0.5, 0.0),
            normal,
            tangent,
            bitangent,
            Vector2::new(1.0, 0.0),
        ),
        Vertex::new_with_bitangent(
            Vector3::new(-0.5, 0.5, 0.0),
            normal,
            tangent,
            bitangent,
            Vector2::new(0.0, 0.0),
        ),
    ];

    MeshDescriptor::new(vertices, vec![0, 1, 2, 0, 2, 3])
}
