//! Engine state resources for ECS integration.
//!
//! Each type in this module represents a piece of engine state that can be
//! stored in the ECS world as a **resource**. The runtime writes into these
//! before running ECS schedules each frame, meaning every system sees a
//! consistent snapshot of frame-computed and event-driven state.
//!
//! All types implement `Component` via the blanket impl in `orbital_ecs`:
//! `impl<T: Any + Debug + Send + Sync> Component for T {}`

use std::sync::Arc;

use cgmath::Vector2;
use hashbrown::HashMap;

// ---------------------------------------------------------------------------
// Frame timing
// ---------------------------------------------------------------------------

/// Number of frames rendered since the application started.
///
/// Updated by the core schedule at the start of every redraw cycle.
/// Starts at 0 and monotonically increases.
#[derive(Debug, Clone, Copy)]
pub struct FrameCounter(pub u64);

/// Time elapsed since the previous frame, in seconds.
///
/// Measured as wall-clock time between consecutive `tick()` calls in the
/// event loop. **Includes** GPU present wait and event processing time
/// from the previous frame. Clamped to `[0.0, 1.0]` to prevent physics
/// explosions after stalls.
///
/// Written by the runtime just before the core schedule runs.
#[derive(Debug, Clone, Copy)]
pub struct DeltaTime(pub f64);

/// Total accumulated time since the application started, in seconds.
///
/// Updated each frame: `TotalTime += DeltaTime`. Unlike `DeltaTime` which
/// is overwritten each frame, this accumulates frame-over-frame.
#[derive(Debug, Clone, Copy)]
pub struct TotalTime(pub f64);

// ---------------------------------------------------------------------------
// Input & window
// ---------------------------------------------------------------------------

/// Current mouse cursor position in window coordinates, as reported by the
/// most recent `CursorMoved` event.
///
/// Written from the winit event handler (at event time), potentially
/// multiple times between redraws. Systems that read this see the *latest*
/// position at the start of the frame.
#[derive(Debug, Clone, Copy)]
pub struct CursorPosition(pub Vector2<f64>);

/// Current window dimensions in logical pixels.
///
/// Written from the `Resized` winit event handler. Updated asynchronously
/// relative to the frame loop.
#[derive(Debug, Clone, Copy)]
pub struct WindowSize(pub Vector2<u32>);

/// Controls whether the mouse cursor is grabbed and hidden on startup.
///
/// Set this resource during `Module::setup()` to have the engine
/// automatically grab the cursor — no need to push `CursorGrabbed` events.
#[derive(Debug, Clone, Copy)]
pub struct CursorGrabConfig(pub bool);

/// A snapshot of the engine's aggregated input state at the start of the
/// current frame.
///
/// Cloned from `AppRuntime::input_state` just before schedules run, so
/// systems see a deterministic input state for the entire frame even if
/// more input events arrive during system execution.
#[derive(Debug, Clone)]
pub struct InputSnapshot(pub orbital_input::InputState);

// ---------------------------------------------------------------------------
// GPU device / queue wrappers
// ---------------------------------------------------------------------------

/// Shared reference to the wgpu [`Device`].
///
/// Wrapped in `Arc` because `Device` implements neither `Clone` nor `Copy`,
/// but systems only need shared access for resource creation / queries
/// (all `Device` methods take `&self`).
#[derive(Debug, Clone)]
pub struct DeviceResource(pub Arc<wgpu::Device>);

/// Shared reference to the wgpu [`Queue`].
///
/// Wrapped in `Arc` — `Queue::write_buffer` and friends take `&self`,
/// so shared access suffices for most use cases.
#[derive(Debug, Clone)]
pub struct QueueResource(pub Arc<wgpu::Queue>);

// ---------------------------------------------------------------------------
// Engine events (replace AppEvent)
// ---------------------------------------------------------------------------

/// Engine-level events that systems can emit and the runtime processes.
///
/// Systems push events into the `EngineEvents` resource during their execution.
/// After all schedules run, the runtime drains and processes them.
#[derive(Debug, Clone)]
pub enum EngineEvent {
    /// Grab or release the mouse cursor.
    CursorGrabbed(bool),
    /// Show or hide the mouse cursor.
    CursorVisible(bool),
    /// Request application closure (graceful exit).
    RequestClose,
    /// Force application closure with an exit code.
    ForceClose { exit_code: i32 },
    /// Request a window redraw.
    RequestRedraw,
}

/// Collection of engine events emitted by systems during the current frame.
///
/// Inserted as an ECS resource. Systems push events via `ResMut<EngineEvents>`.
/// The runtime drains this after schedules finish and processes each event.
#[derive(Debug, Clone, Default)]
pub struct EngineEvents(pub Vec<EngineEvent>);

impl EngineEvents {
    pub fn push(&mut self, event: EngineEvent) {
        self.0.push(event);
    }

    pub fn drain(&mut self) -> Vec<EngineEvent> {
        std::mem::take(&mut self.0)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_events_push_drain() {
        let mut events = EngineEvents::default();
        events.push(EngineEvent::RequestClose);
        events.push(EngineEvent::CursorGrabbed(true));

        let drained = events.drain();
        assert_eq!(drained.len(), 2);
        assert!(events.is_empty());
    }

    #[test]
    fn engine_events_is_empty() {
        let mut events = EngineEvents::default();
        assert!(events.is_empty());

        events.push(EngineEvent::RequestRedraw);
        assert!(!events.is_empty());

        events.drain();
        assert!(events.is_empty());
    }

    #[test]
    fn engine_events_drain_takes_all() {
        let mut events = EngineEvents::default();
        for i in 0..10 {
            events.push(EngineEvent::ForceClose { exit_code: i });
        }
        let drained = events.drain();
        assert_eq!(drained.len(), 10);
        assert!(events.is_empty());
    }
}

// ---------------------------------------------------------------------------
// Shared caches for model realization (mesh + material deduplication)
// ---------------------------------------------------------------------------

/// Shared mesh cache — maps `Arc<MeshDescriptor>` → `Mesh`.
/// Used by the model realization system to avoid re-uploading identical meshes.
/// Wrapped in `Arc` so it can be cloned for use in `Model::from_descriptor`.
pub type MeshCacheResource = Arc<
    std::sync::RwLock<
        orbital_core::cache::Cache<
            std::sync::Arc<orbital_resources::MeshDescriptor>,
            orbital_resources::Mesh,
        >,
    >,
>;

/// Shared material cache — maps `Arc<MaterialShaderDescriptor>` → `MaterialShader`.
/// Used by the model realization system to avoid re-creating identical materials.
/// Wrapped in `Arc` so it can be cloned for use in `Model::from_descriptor`.
pub type MaterialCacheResource = Arc<
    std::sync::RwLock<
        orbital_core::cache::Cache<
            std::sync::Arc<orbital_resources::MaterialShaderDescriptor>,
            orbital_resources::MaterialShader,
        >,
    >,
>;

/// Current surface texture format (set on resume/resize).
#[derive(Debug, Clone, Copy)]
pub struct SurfaceFormatResource(pub wgpu::TextureFormat);

/// Unified light GPU buffer — all lights packed into a single storage buffer.
/// Rebuilt by `realize_lights` when any light is dirty.
#[derive(Debug, Clone)]
pub struct LightBufferResource(pub Option<Arc<wgpu::Buffer>>);

/// Current world environment descriptor (singleton).
/// Set by the environment system when the user changes the HDRI/skybox.
#[derive(Debug, Clone)]
pub struct EnvironmentDescriptorResource(pub Option<orbital_resources::WorldEnvironmentDescriptor>);

/// Realized world environment GPU state (IBL textures, skybox).
/// Created by `realize_environment` from the descriptor.
#[derive(Debug, Clone)]
pub struct EnvironmentGpuResource(pub Option<Arc<orbital_resources::WorldEnvironment>>);

/// GPU camera store — flat Vec indexed by entity.index.
/// CameraRealization on entities holds the index into this store.
/// This avoids the temporary-borrow problem with get_component_store.
pub struct EcsCameraStore {
    cameras: Vec<Option<Arc<std::sync::RwLock<orbital_resources::Camera>>>>,
}

impl EcsCameraStore {
    pub fn new() -> Self {
        Self {
            cameras: Vec::new(),
        }
    }

    pub fn insert(
        &mut self,
        entity_idx: usize,
        camera: Arc<std::sync::RwLock<orbital_resources::Camera>>,
    ) -> usize {
        if entity_idx >= self.cameras.len() {
            self.cameras.resize_with(entity_idx + 1, || None);
        }
        self.cameras[entity_idx] = Some(camera);
        entity_idx
    }

    pub fn get(
        &self,
        entity_idx: usize,
    ) -> Option<&Arc<std::sync::RwLock<orbital_resources::Camera>>> {
        self.cameras.get(entity_idx)?.as_ref()
    }

    pub fn remove(&mut self, entity_idx: usize) {
        if let Some(slot) = self.cameras.get_mut(entity_idx) {
            *slot = None;
        }
    }
}

impl Default for EcsCameraStore {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for EcsCameraStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EcsCameraStore")
            .field("len", &self.cameras.len())
            .finish()
    }
}

/// IBL BRDF lookup texture — generated once, reused every frame.
/// Stored as the IblBrdf generator itself so we can borrow the texture ref.
pub struct IblBrdfResource(pub Option<orbital_resources::IblBrdf>);

impl Clone for IblBrdfResource {
    fn clone(&self) -> Self {
        Self(None)
    }
}

impl std::fmt::Debug for IblBrdfResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IblBrdfResource")
            .field("has_brdf", &self.0.is_some())
            .finish()
    }
}

/// GPU culling state managed by the culling system.
///
/// `Some` holds the [`CullResources`] instance (GPU buffers + pipelines)
/// after the first frame of culling. `None` means culling is not active.
///
/// The renderer reads from this resource to issue indirect draws.
#[derive(Debug)]
pub struct CullResource(pub Option<orbital_resources::CullResources>);

/// Queue of pending import tasks (glTF files to load).
#[derive(Debug, Default)]
pub struct ImportQueueResource(pub Vec<orbital_importer_gltf::ImportTask>);

impl ImportQueueResource {
    pub fn push(&mut self, task: orbital_importer_gltf::ImportTask) {
        self.0.push(task);
    }
}

/// Results from completed imports, ready to be spawned as ECS entities.
#[derive(Debug, Default)]
pub struct ImportResultsResource(pub Vec<orbital_importer_gltf::ImportResult>);

/// The glTF importer — owns the rayon thread pool and mpsc channels.
/// Inserted as an ECS resource by the module during setup.
pub struct ImporterResource(pub orbital_importer_gltf::Importer);

impl ImporterResource {
    pub fn new(allowed_parallel_tasks: u8) -> Self {
        Self(orbital_importer_gltf::Importer::new(allowed_parallel_tasks))
    }
}

// SAFETY: Importer wraps Mutex<Receiver> + Sender + ThreadPool, all Send+Sync.
unsafe impl Send for ImporterResource {}
unsafe impl Sync for ImporterResource {}

impl std::fmt::Debug for ImporterResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImporterResource").finish()
    }
}

/// The frustum data captured when freezing.
///
/// See [`FrozenFrustum`].
#[derive(Debug, Clone)]
pub struct FrozenFrustumData {
    pub frustum: orbital_resources::Frustum,
    /// Stored so the debug overlay can draw the frozen frustum wireframe
    /// without recomputing it from the planes.
    pub perspective_view_projection_matrix: cgmath::Matrix4<f32>,
}

/// A frozen frustum captured at a specific camera position.
///
/// When `Some`, [`sys_frustum_cull`] uses it instead of the live camera
/// frustum. This lets you move the camera around and see exactly which
/// instances are culled.
///
/// Press F4 (in the main loop) to toggle capture.
#[derive(Debug, Clone)]
pub struct FrozenFrustum(pub Option<FrozenFrustumData>);

// ---------------------------------------------------------------------------
// Light scheduling & shadow caching
// ---------------------------------------------------------------------------

/// Maximum number of lights in the GPU storage buffer.
/// Buffer is pre-allocated at MAX_LIGHTS * 64 bytes.
pub const MAX_LIGHTS: u32 = 256;

/// Default maximum number of shadow-casting lights to update per frame.
/// Configurable via [`StaggeredLightConfig`].
pub const DEFAULT_MAX_UPDATES_PER_FRAME: u32 = 1;

/// Tracks stable slot assignments for light entities in the GPU buffer.
///
/// When a light is first realized, it gets assigned the next free slot.
/// When removed, its slot is returned to the free list and the slot is
/// zeroed out (intensity=0) to disable it in the shader.
#[derive(Debug, Clone)]
pub struct LightSlotTracker {
    pub entity_to_slot: Vec<Option<u32>>,
    pub free_slots: Vec<u32>,
    pub slot_count: u32,
}

impl LightSlotTracker {
    pub fn new() -> Self {
        Self {
            entity_to_slot: Vec::new(),
            free_slots: Vec::new(),
            slot_count: 0,
        }
    }

    pub fn allocate(&mut self, entity_id: usize) -> u32 {
        if let Some(slot) = self.free_slots.pop() {
            self.ensure_entity_idx(entity_id);
            self.entity_to_slot[entity_id] = Some(slot);
            slot
        } else {
            let slot = self.slot_count;
            self.slot_count += 1;
            self.ensure_entity_idx(entity_id);
            self.entity_to_slot[entity_id] = Some(slot);
            slot
        }
    }

    pub fn free(&mut self, entity_id: usize) {
        if entity_id < self.entity_to_slot.len()
            && let Some(slot) = self.entity_to_slot[entity_id].take() {
                self.free_slots.push(slot);
            }
    }

    pub fn get(&self, entity_id: usize) -> Option<u32> {
        self.entity_to_slot.get(entity_id).copied().flatten()
    }

    fn ensure_entity_idx(&mut self, entity_id: usize) {
        if entity_id >= self.entity_to_slot.len() {
            self.entity_to_slot.resize(entity_id + 1, None);
        }
    }
}

impl Default for LightSlotTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Cached shadow-casting light state used for change detection.
///
/// Before rendering shadow maps, the current light state is compared
/// against this cache. If nothing changed, the shadow map is reused.
#[derive(Debug, Clone)]
pub struct ShadowCachedLightState {
    pub position: cgmath::Vector3<f32>,
    pub direction: cgmath::Vector3<f32>,
    pub light_type: u32,
    pub outer_cone_angle: f32,
    pub bias: f32,
    pub cascade_count: u32,
    pub cascade_split_lambda: f32,
}

/// Range of shadow slots occupied by a single light.
///
/// Point lights consume 1 slot (with 6 cube faces handled internally),
/// directional lights consume `cascade_count` slots (CSM),
/// spot lights consume 1 slot.
#[derive(Debug, Clone, Copy)]
pub struct ShadowSlotRange {
    pub first_slot: u32,
    pub slot_count: u32,
    pub first_cube: u32,
}

/// Cross-frame cache of shadow slot assignments.
///
/// Maps `light_slot_index` → which shadow slots (and cube layers) the
/// light occupies. Also stores the last-known light state for change
/// detection — if nothing changed, the shadow map is reused without
/// re-rendering.
#[derive(Debug, Clone)]
pub struct ShadowMapCache {
    pub light_to_slots: HashMap<u32, ShadowSlotRange>,
    pub last_state: HashMap<u32, ShadowCachedLightState>,
}

impl ShadowMapCache {
    pub fn new() -> Self {
        Self {
            light_to_slots: HashMap::new(),
            last_state: HashMap::new(),
        }
    }
}

impl Default for ShadowMapCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Configures the per-frame budget for staggered light and shadow updates.
///
/// Insert as an ECS resource. Set `max_updates_per_frame` to control how
/// many shadow-casting lights are processed each frame. Remaining dirty
/// lights are deferred to subsequent frames.
#[derive(Debug, Clone)]
pub struct StaggeredLightConfig {
    pub max_updates_per_frame: u32,
}

impl Default for StaggeredLightConfig {
    fn default() -> Self {
        Self {
            max_updates_per_frame: DEFAULT_MAX_UPDATES_PER_FRAME,
        }
    }
}

/// Round-robin cursor for staggered light/shadow update processing.
/// Managed internally by the stagger system.
#[derive(Debug, Clone, Default)]
pub struct StaggerState {
    pub round_robin_pos: usize,
    pub dirty_queue: Vec<usize>,
}

/// Set by `realize_lights` when new shadow-casting lights are created.
/// The module runtime reads this to inform the stagger system it should
/// perform a full bootstrap pass (all shadows dirty) that frame.
#[derive(Debug, Clone, Copy)]
#[derive(Default)]
pub struct NewLightBootstrap(pub bool);

