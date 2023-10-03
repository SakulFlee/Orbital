use std::f32::consts::FRAC_PI_2;

use cgmath::{InnerSpace, Matrix4, Point3, Rad, Vector3};

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, ShaderStages,
};

use crate::engine::LogicalDevice;

mod u_camera;
pub use u_camera::*;

mod camera_change;
pub use camera_change::*;

mod projection;
pub use projection::*;

#[derive(Debug)]
pub struct Camera {
    position: Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
    speed: f32,
    sensitivity: f32,
    projection: Projection,
    buffer: Buffer,
    bind_group: BindGroup,
}

impl Camera {
    #[rustfmt::skip]
    pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.5,
        0.0, 0.0, 0.0, 1.0,
    );

    pub const DEFAULT_CAMERA_EYE_POSITION: (f32, f32, f32) = (0.0, 1.0, 2.0);

    const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

    pub const BIND_GROUP_LAYOUT_DESCRIPTOR: BindGroupLayoutDescriptor<'static> =
        BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        };

    pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        logical_device: &LogicalDevice,
        position: V,
        yaw: Y,
        pitch: P,
        speed: f32,
        sensitivity: f32,
        projection: Projection,
    ) -> Self {
        let empty_uniform = UCamera::empty();
        let buffer = logical_device
            .device()
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[empty_uniform]),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

        let bind_group_layout = Self::bind_group_layout(logical_device);

        let bind_group = logical_device
            .device()
            .create_bind_group(&BindGroupDescriptor {
                label: Some("Camera Bind Group"),
                layout: &bind_group_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });

        let mut camera = Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
            speed,
            sensitivity,
            projection,
            bind_group,
            buffer,
        };

        camera.update_buffer(logical_device);

        camera
    }

    pub fn calculate_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        Matrix4::look_to_rh(
            self.position,
            Vector3 {
                x: cos_pitch * cos_yaw,
                y: sin_pitch,
                z: cos_pitch * sin_yaw,
            }
            .normalize(),
            Vector3::unit_y(),
        )
    }

    pub fn update_buffer(&mut self, logical_device: &LogicalDevice) {
        let uniform = UCamera::from_camera(self);

        // Write uniform into buffer
        logical_device
            .queue()
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[uniform]))
    }

    pub fn apply_camera_change(
        &mut self,
        delta_time: f64,
        logical_device: &LogicalDevice,
        camera_change: CameraChange,
    ) {
        let (yaw_sin, yaw_cos) = self.yaw.0.sin_cos();

        // Move forward/backward
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        self.position += forward
            * (camera_change.amount_forward() - camera_change.amount_backward())
            * self.speed
            * delta_time as f32;

        // Move left/right
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        self.position += right
            * (camera_change.amount_right() - camera_change.amount_left())
            * self.speed
            * delta_time as f32;

        // Move up/down
        self.position.y += (camera_change.amount_up() - camera_change.amount_down())
            * self.speed
            * delta_time as f32;

        // Rotation
        let half_width = (self.projection().width() / 2) as f32;
        let half_height = (self.projection().height() / 2) as f32;
        self.yaw += Rad(camera_change.rotate_horizontal() - half_width)
            * self.sensitivity
            * delta_time as f32;
        self.pitch += Rad(half_height - camera_change.rotate_vertical())
            * self.sensitivity
            * delta_time as f32;

        // Keep the camera's angle from going too high/low.
        if self.pitch < -Rad(Self::SAFE_FRAC_PI_2) {
            self.pitch = -Rad(Self::SAFE_FRAC_PI_2);
        } else if self.pitch > Rad(Self::SAFE_FRAC_PI_2) {
            self.pitch = Rad(Self::SAFE_FRAC_PI_2);
        }

        self.update_buffer(logical_device);
    }

    pub fn bind_group_layout(logical_device: &LogicalDevice) -> BindGroupLayout {
        logical_device
            .device()
            .create_bind_group_layout(&Self::BIND_GROUP_LAYOUT_DESCRIPTOR)
    }

    pub fn position(&self) -> Point3<f32> {
        self.position
    }

    pub fn set_position(&mut self, position: Point3<f32>) {
        self.position = position;
    }

    pub fn yaw(&self) -> Rad<f32> {
        self.yaw
    }

    pub fn set_yaw(&mut self, yaw: Rad<f32>) {
        self.yaw = yaw;
    }

    pub fn pitch(&self) -> Rad<f32> {
        self.pitch
    }

    pub fn set_pitch(&mut self, pitch: Rad<f32>) {
        self.pitch = pitch;
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    pub fn sensitivity(&self) -> f32 {
        self.sensitivity
    }

    pub fn set_sensitivity(&mut self, sensitivity: f32) {
        self.sensitivity = sensitivity;
    }

    pub fn projection(&self) -> &Projection {
        &self.projection
    }

    pub fn set_projection(&mut self, projection: Projection) {
        self.projection = projection;
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}
