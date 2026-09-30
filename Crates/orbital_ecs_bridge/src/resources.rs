//! Engine state resources for ECS integration.
//!
//! Each type in this module represents a piece of engine state that can be
//! stored in the ECS world as a **resource**. The runtime writes into these
//! before running ECS schedules each frame, meaning every system sees a
//! consistent snapshot of frame-computed and event-driven state.
//!
//! All types implement `Component` via the blanket impl in `orbital_ecs`:
//! `impl<T: Any + Debug + Send + Sync> Component for T {}`

use std::collections::VecDeque;
use std::sync::Arc;

use cgmath::Vector2;
use hashbrown::HashMap;
use orbital_core::logging::info;

// ---------------------------------------------------------------------------
// Frame timing
// ---------------------------------------------------------------------------

/// Number of frames rendered since the application started.
///
/// Updated by the core schedule at the start of every redraw cycle.
/// Starts at 0 and monotonically increases.
#[derive(Debug, Clone, Copy)]
pub struct FrameCounter(pub u64);

/// Frame-rate statistics updated every second by the runtime.
///
/// Written by `ModuleRuntime::update()` alongside the `info!()` log line.
/// Read by HUD overlays (e.g. the all-in-one template) to display live
/// performance metrics on screen.
#[derive(Debug, Clone, Copy)]
pub struct FpsStats {
    /// Frames rendered in the last complete one-second window.
    pub fps: u64,
    /// Cumulative delta time for the current one-second window (seconds).
    pub total_delta_time: f64,
    /// Delta time of the most recent frame (seconds).
    pub cycle_delta_time: f64,
}

impl Default for FpsStats {
    fn default() -> Self {
        Self {
            fps: 0,
            total_delta_time: 0.0,
            cycle_delta_time: 0.0,
        }
    }
}

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

/// One measured stage of the render frame.
///
/// The runtime measures a fixed set of stages between surface acquisition and
/// present; each becomes one `TimingSample` in [`FrameTimings`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimingSample {
    /// Stable identifier for the stage, e.g. `"render"`.
    pub name: &'static str,
    /// The most recent frame's duration, in milliseconds.
    pub last_ms: f64,
    /// Mean duration over the accumulation window, in milliseconds.
    pub avg_ms: f64,
    /// Smallest duration seen in the current window, in milliseconds.
    pub min_ms: f64,
    /// Largest duration seen in the current window, in milliseconds.
    pub max_ms: f64,
}

/// The stage identifiers the runtime measures, in the order they occur in a
/// frame. The GPU stages are derived from three timestamp query results read
/// back from the renderer, and are only meaningful on backends that support
/// timestamp queries; elsewhere they stay at zero.
pub const TIMING_STAGES: [&str; 11] = [
    "surface_acq",
    "realize",
    "stagger",
    "cull+extract",
    "bind+models",
    "render",
    "present",
    "total",
    "gpu shadow",
    "gpu skybox+models",
    "gpu total",
];

/// The index of the `total` stage, which is the whole-frame duration and the
/// series a frame-time graph should plot.
pub const TOTAL_STAGE_INDEX: usize = 7;

impl TimingSample {
    /// A zeroed sample for `index` into [`TIMING_STAGES`].
    ///
    /// # Panics
    /// If `index` is out of bounds.
    pub fn empty(index: usize) -> Self {
        Self {
            name: TIMING_STAGES[index],
            last_ms: 0.0,
            avg_ms: 0.0,
            min_ms: 0.0,
            max_ms: 0.0,
        }
    }
}

/// Per-stage frame timings plus a rolling history, for debug overlays.
///
/// The runtime writes this resource at the end of every redraw, right after
/// present. Unlike the `info!` timing line it replaces, the statistics are
/// *not* reset on print: `avg_ms`/`min_ms`/`max_ms` keep accumulating until
/// [`Self::clear`] is called, and a bounded history is retained so a graph can
/// plot recent frames.
#[derive(Debug, Clone)]
pub struct FrameTimings {
    samples: Vec<TimingSample>,
    /// Running totals backing `avg_ms`, reset by [`Self::clear`].
    totals: Vec<f64>,
    count: u64,
    /// Whole-frame duration per frame, oldest first.
    frame_ms: VecDeque<f32>,
    /// Per-stage durations over time, oldest first. One entry per frame, each
    /// holding one value per stage in [`TIMING_STAGES`] order.
    history: VecDeque<Vec<f64>>,
    /// Totals for the current one-second log window only. Independent of
    /// `totals`, so emitting the summary never disturbs the overlay.
    window_totals: Vec<f64>,
    window_count: u64,
    last_print: std::time::Instant,
}

impl FrameTimings {
    /// How many frames of history to keep.
    pub const HISTORY_LEN: usize = 240;

    /// How often the `info!` timing summary is emitted, in seconds.
    const LOG_INTERVAL_SECS: f64 = 1.0;

    /// A fresh, empty set of timings.
    pub fn new() -> Self {
        let stages = TIMING_STAGES.len();

        Self {
            samples: (0..stages).map(TimingSample::empty).collect(),
            totals: vec![0.0; stages],
            count: 0,
            frame_ms: VecDeque::with_capacity(Self::HISTORY_LEN),
            history: VecDeque::with_capacity(Self::HISTORY_LEN),
            window_totals: vec![0.0; stages],
            window_count: 0,
            last_print: std::time::Instant::now(),
        }
    }

    /// The per-stage samples, in frame order.
    pub fn samples(&self) -> &[TimingSample] {
        &self.samples
    }

    /// The number of frames accumulated since the last [`Self::clear`].
    pub fn frame_count(&self) -> u64 {
        self.count
    }

    /// Whole-frame duration per frame, oldest first.
    pub fn frame_ms(&self) -> &VecDeque<f32> {
        &self.frame_ms
    }

    /// Per-stage durations over time, oldest first.
    pub fn history(&self) -> &VecDeque<Vec<f64>> {
        &self.history
    }

    /// Records one frame's worth of stage durations, in [`TIMING_STAGES`]
    /// order.
    ///
    /// `durations` must have exactly [`TIMING_STAGES`] entries; anything else
    /// is ignored, so a mismatch in the runtime can't corrupt the history.
    pub fn record(&mut self, durations: &[f64]) {
        if durations.len() != self.samples.len() {
            return;
        }

        self.count += 1;
        self.window_count += 1;

        // Seed min/max from the first sample of the window. Starting them at
        // 0.0 would pin `min_ms` at 0 forever, since durations are never
        // negative.
        let first_in_window = self.count == 1;

        for (index, &duration) in durations.iter().enumerate() {
            self.totals[index] += duration;
            self.window_totals[index] += duration;

            let sample = &mut self.samples[index];
            sample.last_ms = duration;

            if first_in_window {
                sample.min_ms = duration;
                sample.max_ms = duration;
            } else {
                sample.min_ms = sample.min_ms.min(duration);
                sample.max_ms = sample.max_ms.max(duration);
            }
        }

        let n = self.count as f64;
        for (index, sample) in self.samples.iter_mut().enumerate() {
            sample.avg_ms = self.totals[index] / n;
        }

        if self.frame_ms.len() == Self::HISTORY_LEN {
            self.frame_ms.pop_front();
        }
        self.frame_ms.push_back(durations[TOTAL_STAGE_INDEX] as f32);

        if self.history.len() == Self::HISTORY_LEN {
            self.history.pop_front();
        }
        self.history.push_back(durations.to_vec());
    }

    /// Resets the accumulating statistics, keeping the history and `last_ms`.
    ///
    /// `avg_ms`/`min_ms`/`max_ms` are re-derived from the first frame recorded
    /// after this call, so they read as zero in the meantime.
    pub fn clear(&mut self) {
        self.totals.iter_mut().for_each(|total| *total = 0.0);
        self.count = 0;
        self.samples.iter_mut().for_each(|sample| {
            sample.avg_ms = 0.0;
            sample.min_ms = 0.0;
            sample.max_ms = 0.0;
        });
    }

    /// Emits the `info!` timing summary at most once per second, returning the
    /// per-stage averages over that window.
    ///
    /// The accumulating statistics are untouched, so the summary and a debug
    /// overlay reading [`Self::samples`] never interfere.
    pub fn try_log(&mut self) -> Option<Vec<f64>> {
        if self.window_count == 0
            || self.last_print.elapsed().as_secs_f64() < Self::LOG_INTERVAL_SECS
        {
            return None;
        }

        let n = self.window_count as f64;
        let averages = self
            .window_totals
            .iter()
            .map(|total| total / n)
            .collect::<Vec<_>>();

        info!(
            "TIMING avg({} frames): surface_acq={:.2}ms realize={:.2}ms stagger={:.2}ms cull+extract={:.2}ms bind+models={:.2}ms render={:.2}ms present={:.2}ms TOTAL={:.2}ms | GPU shadow={:.2}ms skybox+models={:.2}ms GPU_TOTAL={:.2}ms",
            self.window_count,
            averages[0],
            averages[1],
            averages[2],
            averages[3],
            averages[4],
            averages[5],
            averages[6],
            averages[7],
            averages[8],
            averages[9],
            averages[10],
        );

        self.window_totals.iter_mut().for_each(|total| *total = 0.0);
        self.window_count = 0;
        self.last_print = std::time::Instant::now();

        Some(averages)
    }
}

impl Default for FrameTimings {
    fn default() -> Self {
        Self::new()
    }
}

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

/// Live cursor grab state — updated by [`CursorToggle`](orbital_app::systems::CursorToggle)
/// and read by the camera controller to skip mouse rotation when the cursor is free.
#[derive(Debug, Clone, Copy)]
pub struct CursorGrabState(pub bool);

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

/// Shared reference to the wgpu [`Adapter`].
///
/// Needed by iced's `Engine::new()` to construct its renderer.
#[derive(Debug, Clone)]
pub struct AdapterResource(pub Arc<wgpu::Adapter>);

/// An owned window event relevant to iced UI processing.
///
/// Stored in [`IcedEventQueue`] so the iced overlay can consume events
/// without lifetime issues from `winit::event::WindowEvent<'_>`.
#[derive(Debug, Clone)]
pub enum IcedWindowEvent {
    CursorMoved {
        position: winit::dpi::PhysicalPosition<f64>,
    },
    MouseInput {
        state: winit::event::ElementState,
        button: winit::event::MouseButton,
    },
    KeyboardInput {
        event: winit::event::KeyEvent,
        is_synthetic: bool,
    },
    ModifiersChanged(winit::keyboard::ModifiersState),
    Resized(winit::dpi::PhysicalSize<u32>),
    Focused(bool),
    RedrawRequested,
    /// A touch event. On Android (and other touch-only platforms) winit emits
    /// *only* `WindowEvent::Touch` — no synthetic mouse events — so this is the
    /// sole pointer input iced can consume there.
    Touch(winit::event::Touch),
}

/// Queue of winit events to be forwarded to iced UI overlays.
///
/// Populated each frame by `module_runtime.rs` before overlay rendering.
/// Consumed by `IcedLayerRenderer::render()` and drained.
pub struct IcedEventQueue {
    pub events: Vec<IcedWindowEvent>,
    pub cursor_position: Option<winit::dpi::PhysicalPosition<f64>>,
    pub modifiers: winit::keyboard::ModifiersState,
    pub scale_factor: f64,
}

impl Default for IcedEventQueue {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            cursor_position: None,
            modifiers: winit::keyboard::ModifiersState::empty(),
            scale_factor: 1.0,
        }
    }
}

impl IcedEventQueue {
    pub fn push(&mut self, event: IcedWindowEvent) {
        match &event {
            IcedWindowEvent::CursorMoved { position } => {
                self.cursor_position = Some(*position);
            }
            // Like `iced_winit::window::State::update`, a touch updates the
            // cursor position too — otherwise the cursor stays `Unavailable`
            // on touch-only platforms and *all* hit-testing fails.
            IcedWindowEvent::Touch(touch) => {
                self.cursor_position = Some(touch.location);
            }
            IcedWindowEvent::ModifiersChanged(mods) => {
                self.modifiers = *mods;
            }
            _ => {}
        }
        self.events.push(event);
    }

    pub fn set_scale_factor(&mut self, scale: f64) {
        self.scale_factor = scale;
    }

    pub fn drain(&mut self) -> Vec<IcedWindowEvent> {
        std::mem::take(&mut self.events)
    }
}

/// Winit touch ids currently **captured** by an iced UI overlay.
///
/// Populated by the iced overlay renderers (`orbital_iced`): whenever a
/// `WindowEvent::Touch` is converted and handed to iced and the widget tree
/// consumes it (`event::Status::Captured` — e.g. a button press, a
/// `FloatingPanel` title-bar drag or a slider), the touch id is inserted
/// here. Lifted/lost fingers are removed again so stale ids cannot
/// accumulate.
///
/// `module_runtime.rs` consults this resource *before* feeding touch events
/// into the engine's game-input path (`orbital_input::InputState`), so that
/// touches interacting with UI panels don't also drive the virtual joystick
/// or the drag-to-look camera. The iced event queue itself keeps receiving
/// every touch event regardless — capture only masks the game-input path.
///
/// Mirrors [`IcedEventQueue`]: populated during the render pass (iced
/// consumes events *while* rendering), which means capture information for a
/// freshly pressed finger is one frame late. The consumer compensates by
/// synthesizing a `TouchPhase::Cancelled` for the finger once, fully
/// releasing it from the game-input state.
#[derive(Debug, Clone, Default)]
pub struct IcedCapturedTouches(pub hashbrown::HashSet<u64>);

impl IcedCapturedTouches {
    /// Whether the given winit touch id is currently captured by the UI.
    pub fn contains(&self, touch_id: u64) -> bool {
        self.0.contains(&touch_id)
    }

    /// Mark a touch id as captured by the UI. Returns `true` if the id was
    /// newly inserted (i.e. it wasn't captured already).
    pub fn capture(&mut self, touch_id: u64) -> bool {
        self.0.insert(touch_id)
    }

    /// Release a touch id (finger lifted/lost/cancelled).
    pub fn release(&mut self, touch_id: u64) {
        self.0.remove(&touch_id);
    }

    /// Whether any finger is currently captured by the UI.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Whether the current mouse-button drag is owned by the iced UI.
///
/// Set when iced reports `Status::Captured` for a primary-button press
/// (the press started on a widget), cleared when the button is released
/// or focus is lost. The mouse-event path itself is not masked — the
/// camera controller reads this to keep drag-to-look from also firing
/// while the user is dragging a UI element.
///
/// Mirrors [`IcedCapturedTouches`]: populated during
/// `process_events()`. Unlike touch capture there is no need to defer
/// game input — the camera system runs after `process_events()` within
/// the same update, so the verdict is always current when look deltas
/// are consumed.
#[derive(Debug, Clone, Default)]
pub struct IcedCapturedMouseDrag(pub bool);

impl IcedCapturedMouseDrag {
    /// Mark the current mouse drag as captured by the UI.
    pub fn capture(&mut self) {
        self.0 = true;
    }

    /// End capture (button released or focus lost).
    pub fn release(&mut self) {
        self.0 = false;
    }
}

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

    /// A frame whose CPU stages each took `cpu_total`, with no GPU timestamps.
    fn frame(cpu_total: f64) -> Vec<f64> {
        let mut durations = vec![cpu_total; 8];
        durations.resize(TIMING_STAGES.len(), 0.0);
        durations[TOTAL_STAGE_INDEX] = cpu_total;
        durations
    }

    #[test]
    fn frame_timings_starts_zeroed_with_named_stages() {
        let timings = FrameTimings::new();

        assert_eq!(timings.samples().len(), TIMING_STAGES.len());
        assert_eq!(timings.frame_count(), 0);
        assert!(timings.frame_ms().is_empty());
        assert!(timings.history().is_empty());

        for (index, sample) in timings.samples().iter().enumerate() {
            assert_eq!(sample.name, TIMING_STAGES[index]);
            assert_eq!(sample.last_ms, 0.0);
            assert_eq!(sample.avg_ms, 0.0);
        }
    }

    #[test]
    fn frame_timings_tracks_last_avg_min_max() {
        let mut timings = FrameTimings::new();

        for value in [2.0, 4.0, 6.0] {
            timings.record(&frame(value));
        }

        assert_eq!(timings.frame_count(), 3);

        for index in 0..7 {
            let sample = timings.samples()[index];
            assert_eq!(sample.last_ms, 6.0, "stage {index}");
            assert_eq!(sample.avg_ms, 4.0, "stage {index}");
            assert_eq!(sample.min_ms, 2.0, "stage {index}");
            assert_eq!(sample.max_ms, 6.0, "stage {index}");
        }

        // The GPU stages were left at zero because no timestamps were supplied.
        for index in 8..TIMING_STAGES.len() {
            let sample = timings.samples()[index];
            assert_eq!(sample.last_ms, 0.0, "stage {index}");
            assert_eq!(sample.max_ms, 0.0, "stage {index}");
        }
    }

    #[test]
    fn frame_timings_ignores_malformed_frames() {
        let mut timings = FrameTimings::new();

        timings.record(&[1.0, 2.0]);
        timings.record(&vec![1.0; TIMING_STAGES.len() + 1]);
        timings.record(&[]);

        assert_eq!(timings.frame_count(), 0);
        assert!(timings.history().is_empty());
    }

    #[test]
    fn frame_timings_history_is_capped_and_oldest_first() {
        let mut timings = FrameTimings::new();

        let total = FrameTimings::HISTORY_LEN + 10;
        for index in 0..total {
            timings.record(&frame(index as f64));
        }

        assert_eq!(timings.frame_count(), total as u64);
        assert_eq!(timings.frame_ms().len(), FrameTimings::HISTORY_LEN);
        assert_eq!(timings.history().len(), FrameTimings::HISTORY_LEN);

        // Oldest retained sample is 10 frames back, newest is the last one.
        let frame_ms = timings.frame_ms();
        assert_eq!(*frame_ms.front().expect("non-empty"), 10.0);
        assert_eq!(*frame_ms.back().expect("non-empty"), (total - 1) as f32);

        let history = timings.history();
        assert_eq!(history[0][0], 10.0);
        assert_eq!(history[0].len(), TIMING_STAGES.len());
    }

    #[test]
    fn frame_timings_clear_resets_stats_but_keeps_history() {
        let mut timings = FrameTimings::new();

        timings.record(&frame(2.0));
        timings.record(&frame(4.0));
        assert_eq!(timings.frame_ms().len(), 2);

        timings.clear();

        assert_eq!(timings.frame_count(), 0);
        assert_eq!(timings.frame_ms().len(), 2, "history should survive clear");
        assert_eq!(timings.samples()[0].avg_ms, 0.0);
        assert_eq!(timings.samples()[0].min_ms, 0.0);
        assert_eq!(timings.samples()[0].max_ms, 0.0);
        assert_eq!(timings.samples()[0].last_ms, 4.0, "last is untouched");

        // A fresh window restarts min/max from the first frame in it, rather
        // than carrying the previous window's extremes forward.
        timings.record(&frame(1.0));
        let sample = timings.samples()[0];
        assert_eq!(sample.avg_ms, 1.0);
        assert_eq!(sample.min_ms, 1.0);
        assert_eq!(sample.max_ms, 1.0);

        timings.record(&frame(3.0));
        let sample = timings.samples()[0];
        assert_eq!(sample.avg_ms, 2.0);
        assert_eq!(sample.min_ms, 1.0);
        assert_eq!(sample.max_ms, 3.0);
    }

    #[test]
    fn frame_timings_log_window_is_independent_of_accumulated_stats() {
        let mut timings = FrameTimings::new();

        // Not enough wall-clock time has passed, so nothing is logged yet.
        timings.record(&frame(2.0));
        assert!(timings.try_log().is_none());
        assert_eq!(timings.frame_count(), 1, "logging must not reset the stats");

        // Backdate the window so the next call logs.
        timings.last_print = std::time::Instant::now() - std::time::Duration::from_secs(2);
        let logged = timings.try_log().expect("should log after the interval");

        assert_eq!(logged.len(), TIMING_STAGES.len());
        // The CPU stages were recorded; the GPU ones had no timestamps.
        for value in &logged[..8] {
            assert!(*value > 0.0);
        }
        for value in &logged[8..] {
            assert_eq!(*value, 0.0);
        }

        // Window resets, accumulated statistics do not.
        assert_eq!(timings.window_count, 0);
        assert_eq!(timings.frame_count(), 1);
        assert_eq!(timings.samples()[0].avg_ms, 2.0);

        // And it won't log again until another second has passed.
        assert!(timings.try_log().is_none());
    }

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

    #[test]
    fn iced_captured_touches_lifecycle() {
        let mut captured = IcedCapturedTouches::default();
        assert!(captured.is_empty());
        assert!(!captured.contains(42));

        // A widget captures the finger...
        captured.capture(42);
        assert!(captured.contains(42));
        assert!(!captured.is_empty());

        // ...and the finger is eventually lifted.
        captured.release(42);
        assert!(!captured.contains(42));
        assert!(captured.is_empty());
    }

    #[test]
    fn iced_captured_touches_track_multiple_fingers() {
        let mut captured = IcedCapturedTouches::default();

        captured.capture(1);
        captured.capture(2);
        assert!(captured.contains(1) && captured.contains(2));

        // Releasing one finger must not disturb the other (multi-touch:
        // one finger on the UI while another drives the camera).
        captured.release(1);
        assert!(!captured.contains(1));
        assert!(captured.contains(2));

        // Capturing again is idempotent.
        captured.capture(2);
        assert!(captured.contains(2));
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
            std::sync::Arc<orbital_mesh::MeshDescriptor>,
            orbital_mesh::Mesh,
        >,
    >,
>;

/// Shared material cache — maps `Arc<MaterialShaderDescriptor>` → `MaterialShader`.
/// Used by the model realization system to avoid re-creating identical materials.
/// Wrapped in `Arc` so it can be cloned for use in `Model::from_descriptor`.
pub type MaterialCacheResource = Arc<
    std::sync::RwLock<
        orbital_core::cache::Cache<
            std::sync::Arc<orbital_material_shader::MaterialShaderDescriptor>,
            orbital_material_shader::MaterialShader,
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
pub struct EnvironmentDescriptorResource(
    pub Option<orbital_world_environment::WorldEnvironmentDescriptor>,
);

/// Realized world environment GPU state (IBL textures, skybox).
/// Created by `realize_environment` from the descriptor.
#[derive(Debug, Clone)]
pub struct EnvironmentGpuResource(pub Option<Arc<orbital_world_environment::WorldEnvironment>>);

/// GPU camera store — flat Vec indexed by entity.index.
/// CameraRealization on entities holds the index into this store.
/// This avoids the temporary-borrow problem with get_component_store.
pub struct EcsCameraStore {
    cameras: Vec<Option<Arc<std::sync::RwLock<orbital_camera::Camera>>>>,
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
        camera: Arc<std::sync::RwLock<orbital_camera::Camera>>,
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
    ) -> Option<&Arc<std::sync::RwLock<orbital_camera::Camera>>> {
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
pub struct IblBrdfResource(pub Option<orbital_ibl_brdf::IblBrdf>);

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
pub struct CullResource(pub Option<orbital_cull::CullResources>);

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
    pub frustum: orbital_camera::Frustum,
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
            && let Some(slot) = self.entity_to_slot[entity_id].take()
        {
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
#[derive(Debug, Clone, Copy, Default)]
pub struct NewLightBootstrap(pub bool);
