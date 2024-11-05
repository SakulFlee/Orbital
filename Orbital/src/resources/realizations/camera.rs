use cgmath::{perspective, Deg, InnerSpace, Matrix, Matrix4, SquareMatrix, Vector3};
use std::mem;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutEntry, BindingType, Buffer,
    BufferBindingType, BufferDescriptor, BufferUsages, Device, Queue, ShaderStages,
};

use crate::{
    game::CameraChange,
    resources::descriptors::{CameraDescriptor, PipelineBindGroupLayout},
};

#[derive(Debug)]
pub struct Camera {
    descriptor: CameraDescriptor,
    bind_group: BindGroup,
    buffer: Buffer,
}

impl Camera {
    pub fn pipeline_bind_group_layout() -> PipelineBindGroupLayout {
        PipelineBindGroupLayout {
            label: "Camera",
            entries: vec![BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        }
    }

    pub fn from_descriptor(descriptor: CameraDescriptor, device: &Device, queue: &Queue) -> Self {
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Camera Buffer"),
            size: (
                // We have the following variables in our Buffer:
                // position:                            vec4<f32>   -> 4x f32
                mem::size_of::<f32>() * 3 +
                // view_projection_matrix:              mat4x4<f32> -> 4x4x f32
                mem::size_of::<f32>() * 4 * 4 +
                // perspective_view_projection_matrix:  mat4x4<f32> -> 4x4x f32
                mem::size_of::<f32>() * 4 * 4 +
                // view_projection_transposed:          mat4x4<f32> -> 4x4x f32
                mem::size_of::<f32>() * 4 * 4 +
                // perspective_projection_invert:       mat4x4<f32> -> 4x4x f32
                mem::size_of::<f32>() * 4 * 4 +
                // global_gamma:
                mem::size_of::<f32>() +
                // skybox_gamma:
                mem::size_of::<f32>() +
                // Padding ... This should align the buffer to 288.
                12
            ) as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group_layout = Self::pipeline_bind_group_layout().make_bind_group_layout(device);
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        let mut camera = Self {
            descriptor,
            bind_group,
            buffer,
        };
        camera.update_buffer(queue);
        camera
    }

    pub fn update_from_change(&mut self, change: CameraChange, _device: &Device, queue: &Queue) {
        self.descriptor.apply_change(change);
        self.update_buffer(queue);
    }

    fn update_buffer(&mut self, queue: &Queue) {
        let view_projection_matrix = self.calculate_view_projection_matrix();
        let perspective_projection_matrix = self.calculate_perspective_projection_matrix();

        let perspective_view_projection_matrix =
            perspective_projection_matrix * view_projection_matrix;

        let view_projection_transposed = view_projection_matrix.transpose();
        let perspective_projection_invert = perspective_projection_matrix.invert().unwrap();

        queue.write_buffer(
            &self.buffer,
            0,
            &[
                // Position (+ offset to make vec4)
                self.descriptor.position.x.to_le_bytes(),
                self.descriptor.position.y.to_le_bytes(),
                self.descriptor.position.z.to_le_bytes(),
                [0u8; 4],
                // View Projection Matrix
                view_projection_matrix.x.x.to_le_bytes(),
                view_projection_matrix.x.y.to_le_bytes(),
                view_projection_matrix.x.z.to_le_bytes(),
                view_projection_matrix.x.w.to_le_bytes(),
                view_projection_matrix.y.x.to_le_bytes(),
                view_projection_matrix.y.y.to_le_bytes(),
                view_projection_matrix.y.z.to_le_bytes(),
                view_projection_matrix.y.w.to_le_bytes(),
                view_projection_matrix.z.x.to_le_bytes(),
                view_projection_matrix.z.y.to_le_bytes(),
                view_projection_matrix.z.z.to_le_bytes(),
                view_projection_matrix.z.w.to_le_bytes(),
                view_projection_matrix.w.x.to_le_bytes(),
                view_projection_matrix.w.y.to_le_bytes(),
                view_projection_matrix.w.z.to_le_bytes(),
                view_projection_matrix.w.w.to_le_bytes(),
                // Perspective View Projection Matrix
                perspective_view_projection_matrix.x.x.to_le_bytes(),
                perspective_view_projection_matrix.x.y.to_le_bytes(),
                perspective_view_projection_matrix.x.z.to_le_bytes(),
                perspective_view_projection_matrix.x.w.to_le_bytes(),
                perspective_view_projection_matrix.y.x.to_le_bytes(),
                perspective_view_projection_matrix.y.y.to_le_bytes(),
                perspective_view_projection_matrix.y.z.to_le_bytes(),
                perspective_view_projection_matrix.y.w.to_le_bytes(),
                perspective_view_projection_matrix.z.x.to_le_bytes(),
                perspective_view_projection_matrix.z.y.to_le_bytes(),
                perspective_view_projection_matrix.z.z.to_le_bytes(),
                perspective_view_projection_matrix.z.w.to_le_bytes(),
                perspective_view_projection_matrix.w.x.to_le_bytes(),
                perspective_view_projection_matrix.w.y.to_le_bytes(),
                perspective_view_projection_matrix.w.z.to_le_bytes(),
                perspective_view_projection_matrix.w.w.to_le_bytes(),
                // Transposed View Projection Matrix
                view_projection_transposed.x.x.to_le_bytes(),
                view_projection_transposed.x.y.to_le_bytes(),
                view_projection_transposed.x.z.to_le_bytes(),
                view_projection_transposed.x.w.to_le_bytes(),
                view_projection_transposed.y.x.to_le_bytes(),
                view_projection_transposed.y.y.to_le_bytes(),
                view_projection_transposed.y.z.to_le_bytes(),
                view_projection_transposed.y.w.to_le_bytes(),
                view_projection_transposed.z.x.to_le_bytes(),
                view_projection_transposed.z.y.to_le_bytes(),
                view_projection_transposed.z.z.to_le_bytes(),
                view_projection_transposed.z.w.to_le_bytes(),
                view_projection_transposed.w.x.to_le_bytes(),
                view_projection_transposed.w.y.to_le_bytes(),
                view_projection_transposed.w.z.to_le_bytes(),
                view_projection_transposed.w.w.to_le_bytes(),
                // Inverted Perspective Projection Matrix
                perspective_projection_invert.x.x.to_le_bytes(),
                perspective_projection_invert.x.y.to_le_bytes(),
                perspective_projection_invert.x.z.to_le_bytes(),
                perspective_projection_invert.x.w.to_le_bytes(),
                perspective_projection_invert.y.x.to_le_bytes(),
                perspective_projection_invert.y.y.to_le_bytes(),
                perspective_projection_invert.y.z.to_le_bytes(),
                perspective_projection_invert.y.w.to_le_bytes(),
                perspective_projection_invert.z.x.to_le_bytes(),
                perspective_projection_invert.z.y.to_le_bytes(),
                perspective_projection_invert.z.z.to_le_bytes(),
                perspective_projection_invert.z.w.to_le_bytes(),
                perspective_projection_invert.w.x.to_le_bytes(),
                perspective_projection_invert.w.y.to_le_bytes(),
                perspective_projection_invert.w.z.to_le_bytes(),
                perspective_projection_invert.w.w.to_le_bytes(),
                // Global Gamma
                self.descriptor.global_gamma.to_le_bytes(),
            ]
            .concat(),
        );
    }

    pub fn calculate_view_projection_matrix(&self) -> Matrix4<f32> {
        // Takes yaw and pitch values and converts them into a target vector for our camera.
        let (pitch_sin, pitch_cos) = self.descriptor.pitch.sin_cos();
        let (yaw_sin, yaw_cos) = self.descriptor.yaw.sin_cos();

        // Calculates the view project matrix
        Matrix4::look_to_rh(
            self.descriptor.position,
            Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize(),
            Vector3::unit_y(),
        )
    }

    pub fn calculate_perspective_projection_matrix(&self) -> Matrix4<f32> {
        perspective(
            Deg(self.descriptor.fovy),
            self.descriptor.aspect,
            self.descriptor.near,
            self.descriptor.far,
        )
    }

    pub fn descriptor(&self) -> &CameraDescriptor {
        &self.descriptor
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
}
