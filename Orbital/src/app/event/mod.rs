use std::sync::Arc;

use cgmath::Vector2;
use wgpu::{Device, Queue, SurfaceConfiguration, TextureView};
use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};

mod input;
pub use input::*;

#[derive(Debug)]
pub enum AppEvent {
    Resumed(SurfaceConfiguration, Arc<Device>, Arc<Queue>),
    Suspended,
    InputEvent(InputEvent),
    WindowEvent(WindowEvent),
    NewEventCycle(StartCause),
    DeviceEvent(DeviceId, DeviceEvent),
    MemoryWarning,
    Resize(Vector2<u32>, Arc<Device>, Arc<Queue>),
    Render(TextureView, Arc<Device>, Arc<Queue>),
    Update,
}
