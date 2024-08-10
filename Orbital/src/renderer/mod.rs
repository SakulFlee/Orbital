use cgmath::Vector2;
use wgpu::{Device, Queue, TextureFormat, TextureView};

use crate::resources::{descriptors::MaterialDescriptor, realizations::{Camera, LightStorage, Material, Model}};

pub mod standard;
pub use standard::*;

pub trait Renderer {
    fn new(
        surface_texture_format: TextureFormat,
        resolution: Vector2<u32>,
        device: &Device,
        queue: &Queue,
    ) -> Self;

    fn change_surface_texture_format(
        &mut self,
        surface_texture_format: TextureFormat,
        device: &Device,
        queue: &Queue,
    );

    fn change_resolution(&mut self, resolution: Vector2<u32>, device: &Device, queue: &Queue);

    fn update(&mut self, delta_time: f64);

    // TODO: Change to raw data types (e.g. bind group) to make it more
    // compatible with other implementations :: or world?
    fn render(
        &mut self,
        target_view: &TextureView,
        device: &Device,
        queue: &Queue,
        models: &[&Model],
        camera: &Camera,
        light_storage: &LightStorage,
        sky_box_material: &MaterialDescriptor,
    );
}
