use std::sync::Arc;

use cgmath::Vector2;
use wgpu::{Device, Queue, SurfaceConfiguration, SurfaceTexture, TextureView};

use crate::input::InputState;

#[derive(Debug)]
pub enum AppEvent {
    Resumed(SurfaceConfiguration, Arc<Device>, Arc<Queue>),
    Suspended,
    FocusChange { focused: bool },
    Resize(Vector2<u32>, Arc<Device>, Arc<Queue>),
    Render(SurfaceTexture, TextureView, Arc<Device>, Arc<Queue>),
    Update(InputState),
}
