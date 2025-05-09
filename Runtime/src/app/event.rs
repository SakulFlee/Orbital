use std::sync::Arc;

use cgmath::Vector2;
use message::Message;
use wgpu::{Device, Queue, SurfaceConfiguration, SurfaceTexture, TextureView};

use crate::input::InputState;

#[derive(Debug)]
pub enum AppEvent {
    Resumed(SurfaceConfiguration, Arc<Device>, Arc<Queue>),
    Suspended,
    Resize(Vector2<u32>, Arc<Device>, Arc<Queue>),
    Render(SurfaceTexture, TextureView, Arc<Device>, Arc<Queue>),
    Update {
        input_state: InputState,
        delta_time: f64,
        cycle: Option<(f64, u64)>,
        messages: Vec<Message>,
    },
}
