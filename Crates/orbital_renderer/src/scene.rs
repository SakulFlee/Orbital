//! ECS → renderer extraction.
//!
//! [`RenderScene`] is the boundary between the ECS world and the renderer. It
//! owns cheap `Arc`/`wgpu::Buffer` clones for one frame, so the renderer never
//! reaches into the ECS and never keeps borrowed component-store data alive
//! across a render pass.
//!
//! Animation support will extend this snapshot with per-instance bone
//! matrices and previous-frame transforms. Keeping those additions here means
//! the render passes can stay animation-agnostic.

use std::sync::{Arc, RwLock};

use cgmath::{Matrix4, Point3};
use orbital_camera::Camera;
use orbital_ecs::World;
use orbital_ecs_bridge::{
    ActiveCamera, CameraStore, EnvironmentGpuResource, LightBufferResource, LightDescriptor,
    LightSlotIndex, ModelRealization, Position, VolumetricSettings,
};
use orbital_light::LightType;
use orbital_model::Model;
use orbital_shadow::{ShadowCaster, ShadowLightInfo};
use orbital_world_environment::WorldEnvironment;
use wgpu::Buffer;

/// A camera snapshot for one view.
///
/// The buffer is a cheap clone of the camera's uniform buffer (wgpu buffers are
/// reference-counted). The matrices are copied so the renderer does not have to
/// hold the camera lock while recording a frame.
#[derive(Debug)]
pub struct RenderCamera {
    /// The realized camera. Kept alive so `source_id` stays unique for the
    /// lifetime of the bind-group cache.
    source: Arc<RwLock<Camera>>,
    /// GPU uniform buffer with the current camera data.
    pub buffer: Buffer,
    /// Perspective view-projection matrix, used for shadow cascades.
    pub perspective_view_projection: Matrix4<f32>,
    /// World-to-view matrix, used for temporal reprojection.
    pub view_matrix: Matrix4<f32>,
    /// World-space camera position, used to sort transparent draws.
    pub position: Point3<f32>,
    /// Camera near plane.
    pub near: f32,
    /// Camera far plane.
    pub far: f32,
}

impl RenderCamera {
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// Stable identity of the realized camera, used to cache bind groups.
    pub fn source_id(&self) -> usize {
        Arc::as_ptr(&self.source) as usize
    }

    /// Keeps the camera alive while a cached bind group references its buffer.
    pub(crate) fn source(&self) -> Arc<RwLock<Camera>> {
        Arc::clone(&self.source)
    }
}

/// Everything the renderer needs from the ECS for one frame.
///
/// This is deliberately a snapshot: all references are owned (`Arc` or wgpu
/// handles), so it can be handed to the renderer without keeping component
/// stores borrowed.
#[derive(Default)]
pub struct RenderScene {
    /// Active camera, if one has been realized yet.
    pub camera: Option<RenderCamera>,
    /// Unified light storage buffer.
    pub light_buffer: Option<Arc<Buffer>>,
    /// Active world environment (IBL textures + skybox shader).
    pub environment: Option<Arc<WorldEnvironment>>,
    /// Realized models and their instances.
    pub models: Vec<Arc<Model>>,
    /// Shadow-casting lights for this frame.
    pub shadow_lights: Vec<ShadowLightInfo>,
    /// Number of active lights in `light_buffer`.
    pub light_count: u32,
    /// Optional volumetric lighting/fog settings.
    pub volumetrics: Option<VolumetricSettings>,
}

impl RenderScene {
    /// Build a render snapshot from the ECS world.
    pub fn from_world(world: &World) -> Self {
        Self {
            camera: extract_camera(world),
            light_buffer: world
                .get_resource::<LightBufferResource>()
                .and_then(|resource| resource.0.clone()),
            environment: world
                .get_resource::<EnvironmentGpuResource>()
                .and_then(|resource| resource.0.clone()),
            models: collect_models(world),
            shadow_lights: collect_shadow_lights(world),
            light_count: world
                .get_component_store::<LightDescriptor>()
                .map(|store| store.dense.len() as u32)
                .unwrap_or(0),
            volumetrics: world.get_resource::<VolumetricSettings>().map(|s| *s),
        }
    }
}

fn extract_camera(world: &World) -> Option<RenderCamera> {
    let active_camera = world.get_resource::<ActiveCamera>()?;
    let camera_store = world.get_resource::<CameraStore>()?;
    let camera = camera_store.get(active_camera.0.index)?;
    let camera_guard = camera.read().ok()?;

    Some(RenderCamera {
        source: Arc::clone(camera),
        buffer: camera_guard.camera_buffer().clone(),
        perspective_view_projection: *camera_guard.perspective_view_projection_matrix(),
        view_matrix: *camera_guard.view_matrix(),
        position: camera_guard.position(),
        near: camera_guard.near(),
        far: camera_guard.far(),
    })
}

fn collect_models(world: &World) -> Vec<Arc<Model>> {
    let Some(realizations) = world.get_component_store::<ModelRealization>() else {
        return Vec::new();
    };

    realizations
        .dense
        .iter()
        .filter_map(|&entity_id| {
            let index = realizations.sparse.get(entity_id).copied().flatten()?;
            Some(Arc::clone(&realizations.components[index].0))
        })
        .collect()
}

/// Collect shadow-casting light info from ECS.
fn collect_shadow_lights(world: &World) -> Vec<ShadowLightInfo> {
    let Some(descriptors) = world.get_component_store::<LightDescriptor>() else {
        return Vec::new();
    };
    let Some(positions) = world.get_component_store::<Position>() else {
        return Vec::new();
    };
    let Some(casters) = world.get_component_store::<ShadowCaster>() else {
        return Vec::new();
    };

    let slot_indices = world.get_component_store::<LightSlotIndex>();
    let has_slot_store = slot_indices.is_some();

    let mut lights = Vec::new();
    let mut fallback_counter = 0u32;

    for &entity_id in descriptors.dense.as_slice() {
        let Some(descriptor_index) = descriptors.sparse.get(entity_id).copied().flatten() else {
            continue;
        };
        let descriptor = &descriptors.components[descriptor_index];

        let Some(caster_index) = casters.sparse.get(entity_id).copied().flatten() else {
            continue;
        };
        if !casters.components[caster_index].enabled {
            continue;
        }

        let Some(position_index) = positions.sparse.get(entity_id).copied().flatten() else {
            continue;
        };

        let caster = &casters.components[caster_index];
        let position = &positions.components[position_index];
        let (light_type, outer_cone_angle) = match descriptor.light_type {
            LightType::Point { .. } => (0, 0.0),
            LightType::Directional { .. } => (1, 0.0),
            LightType::Spot {
                outer_cone_angle, ..
            } => (2, outer_cone_angle),
        };

        let light_store_index = if has_slot_store {
            slot_indices
                .as_ref()
                .and_then(|store| store.get_component(entity_id))
                .map(|slot| slot.0)
                .unwrap_or(fallback_counter)
        } else {
            fallback_counter
        };
        if !has_slot_store {
            fallback_counter += 1;
        }

        lights.push(ShadowLightInfo {
            light_type,
            direction: descriptor.direction,
            position: cgmath::Vector3::new(position.0.x, position.0.y, position.0.z),
            caster: caster.clone(),
            outer_cone_angle,
            light_store_index,
        });
    }

    lights
}

#[cfg(test)]
mod tests {
    use super::*;
    use orbital_ecs::World;
    use orbital_ecs_bridge::LightDescriptor;

    fn point_light() -> LightDescriptor {
        LightDescriptor::new_point(cgmath::Vector3::new(1.0, 1.0, 1.0), 1.0)
    }

    #[test]
    fn collects_light_count_and_shadow_slot() {
        let mut world = World::new();
        let entity = world.spawn_entity();
        let position = cgmath::Vector3::new(1.0, 2.0, 3.0);

        world.attach_component(&entity, point_light()).unwrap();
        world
            .attach_component(&entity, Position::new(position.x, position.y, position.z))
            .unwrap();
        world
            .attach_component(&entity, ShadowCaster::default())
            .unwrap();
        world.attach_component(&entity, LightSlotIndex(7)).unwrap();

        let scene = RenderScene::from_world(&world);

        assert_eq!(scene.light_count, 1);
        assert_eq!(scene.shadow_lights.len(), 1);
        assert_eq!(scene.shadow_lights[0].light_type, 0);
        assert_eq!(scene.shadow_lights[0].light_store_index, 7);
        assert_eq!(scene.shadow_lights[0].position, position);
    }

    #[test]
    fn disabled_shadow_caster_and_missing_position_are_skipped() {
        let mut world = World::new();

        let disabled = world.spawn_entity();
        world.attach_component(&disabled, point_light()).unwrap();
        world
            .attach_component(&disabled, Position::default())
            .unwrap();
        world
            .attach_component(
                &disabled,
                ShadowCaster {
                    enabled: false,
                    ..Default::default()
                },
            )
            .unwrap();

        let missing_position = world.spawn_entity();
        world
            .attach_component(&missing_position, point_light())
            .unwrap();
        world
            .attach_component(&missing_position, ShadowCaster::default())
            .unwrap();

        let scene = RenderScene::from_world(&world);

        assert_eq!(scene.light_count, 2);
        assert!(scene.shadow_lights.is_empty());
    }
}
