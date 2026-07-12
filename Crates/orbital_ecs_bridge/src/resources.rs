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
// Shared caches (moved from orbital_world::World)
// ---------------------------------------------------------------------------

/// Shared mesh cache — maps `Arc<MeshDescriptor>` → `Mesh`.
/// Used by the model realization system to avoid re-uploading identical meshes.
/// Wrapped in `Arc` so it can be cloned for use in `Model::from_descriptor`.
pub type MeshCacheResource = Arc<std::sync::RwLock<orbital_core::cache::Cache<std::sync::Arc<orbital_resources::MeshDescriptor>, orbital_resources::Mesh>>>;

/// Shared material cache — maps `Arc<MaterialShaderDescriptor>` → `MaterialShader`.
/// Used by the model realization system to avoid re-creating identical materials.
/// Wrapped in `Arc` so it can be cloned for use in `Model::from_descriptor`.
pub type MaterialCacheResource = Arc<std::sync::RwLock<orbital_core::cache::Cache<std::sync::Arc<orbital_resources::MaterialShaderDescriptor>, orbital_resources::MaterialShader>>>;

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
        Self { cameras: Vec::new() }
    }

    pub fn insert(&mut self, entity_idx: usize, camera: Arc<std::sync::RwLock<orbital_resources::Camera>>) -> usize {
        if entity_idx >= self.cameras.len() {
            self.cameras.resize_with(entity_idx + 1, || None);
        }
        self.cameras[entity_idx] = Some(camera);
        entity_idx
    }

    pub fn get(&self, entity_idx: usize) -> Option<&Arc<std::sync::RwLock<orbital_resources::Camera>>> {
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
