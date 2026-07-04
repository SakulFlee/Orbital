mod settings;
pub use settings::*;

mod runtime_event;
pub use runtime_event::*;

mod runtime;
pub use runtime::*;

pub use orbital_element::AppEvent;

mod timer;
pub use timer::*;

mod context;
pub use context::*;

mod state;
pub use state::*;

pub use orbital_input as input;
pub use orbital_input::*;

pub mod standard;

use wgpu::{Device, Queue, SurfaceConfiguration, TextureView};

pub trait App: Send + Sync {
    fn new() -> Self;

    fn on_startup(&mut self) {}

    fn on_resume(&mut self, _config: &SurfaceConfiguration, _device: &Device, _queue: &Queue) {}

    fn on_suspend(&mut self) {}

    fn on_resize(&mut self, _new_size: cgmath::Vector2<u32>, _device: &Device, _queue: &Queue) {}

    fn on_focus_change(&mut self, _focused: bool) {}

    fn on_update(
        &mut self,
        _input_state: &InputState,
        _delta_time: f64,
        _cycle: Option<(f64, u64)>,
    ) -> Option<Vec<AppEvent>> {
        None
    }

    fn on_render(&mut self, _target_view: &TextureView, _device: &Device, _queue: &Queue) {}
}
