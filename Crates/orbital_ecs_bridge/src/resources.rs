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

use orbital_ecs::Entity;

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
// Camera
// ---------------------------------------------------------------------------

/// Marks which entity is the active camera for rendering.
///
/// The renderer reads this resource each frame to determine which
/// camera's view and projection matrices to use.
#[derive(Debug, Clone, Copy)]
pub struct ActiveCamera(pub Entity);
