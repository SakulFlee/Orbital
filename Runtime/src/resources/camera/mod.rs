use std::mem;

use cgmath::{perspective, Deg, InnerSpace, Matrix, Matrix4, SquareMatrix, Vector3, Vector4};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsages,
    Device, Queue, ShaderStages,
};

mod change;
pub use change::*;

mod mode;
pub use mode::*;

mod descriptor;
pub use descriptor::*;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Camera {
    descriptor: CameraDescriptor,
    camera_bind_group: BindGroup,
    camera_buffer: Buffer,
    frustum_bind_group: BindGroup,
    frustum_buffer: Buffer,
}

impl Camera {
    pub fn from_descriptor(descriptor: CameraDescriptor, device: &Device, queue: &Queue) -> Self {
        let camera_buffer = device.create_buffer(&BufferDescriptor {
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
        let camera_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Camera"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::all(),
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let frustum_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Camera Buffer"),
            size: (
                // Left
                mem::size_of::<f32>() * 4 +
                // Right
                mem::size_of::<f32>() * 4 +
                // Top
                mem::size_of::<f32>() * 4 +
                // Bottom
                mem::size_of::<f32>() * 4 +
                // Near
                mem::size_of::<f32>() * 4 +
                // Far
                mem::size_of::<f32>() * 4
            ) as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let frustum_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Frustum"),
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
            });
        let frustum_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Camera Frustum Bind Group"),
            layout: &frustum_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: frustum_buffer.as_entire_binding(),
            }],
        });

        let mut camera = Self {
            descriptor,
            camera_bind_group,
            camera_buffer,
            frustum_bind_group,
            frustum_buffer,
        };
        camera.update_buffers(queue);
        camera
    }

    pub fn update_from_change(&mut self, change: CameraTransform, _device: &Device, queue: &Queue) {
        self.descriptor.apply_change(change);
        self.update_buffers(queue);
    }

    pub fn calculate_frustum_planes(
        &self,
        view_projection_matrix: Option<Matrix4<f32>>,
        perspective_projection_matrix: Option<Matrix4<f32>>,
    ) -> [Vector4<f32>; 6] {
        let view_projection_matrix =
            view_projection_matrix.unwrap_or(self.calculate_view_projection_matrix());
        let perspective_projection_matrix =
            perspective_projection_matrix.unwrap_or(self.calculate_perspective_projection_matrix());

        let perspective_view_projection_matrix =
            perspective_projection_matrix * view_projection_matrix;

        [
            // Left
            (perspective_view_projection_matrix.w + perspective_view_projection_matrix.x)
                .normalize(),
            // Right
            (perspective_view_projection_matrix.w - perspective_view_projection_matrix.x)
                .normalize(),
            // Bottom
            (perspective_view_projection_matrix.w + perspective_view_projection_matrix.y)
                .normalize(),
            // Top
            (perspective_view_projection_matrix.w - perspective_view_projection_matrix.y)
                .normalize(),
            // Near
            (perspective_view_projection_matrix.w + perspective_view_projection_matrix.z)
                .normalize(),
            // Far
            (perspective_view_projection_matrix.w - perspective_view_projection_matrix.z)
                .normalize(),
        ]
    }

    pub fn frustum_to_bytes(frustum_planes: &[Vector4<f32>; 6]) -> [[u8; 4]; 6 * 4] {
        [
            frustum_planes[0].x.to_le_bytes(),
            frustum_planes[0].y.to_le_bytes(),
            frustum_planes[0].z.to_le_bytes(),
            frustum_planes[0].w.to_le_bytes(),
            frustum_planes[1].x.to_le_bytes(),
            frustum_planes[1].y.to_le_bytes(),
            frustum_planes[1].z.to_le_bytes(),
            frustum_planes[1].w.to_le_bytes(),
            frustum_planes[2].x.to_le_bytes(),
            frustum_planes[2].y.to_le_bytes(),
            frustum_planes[2].z.to_le_bytes(),
            frustum_planes[2].w.to_le_bytes(),
            frustum_planes[3].x.to_le_bytes(),
            frustum_planes[3].y.to_le_bytes(),
            frustum_planes[3].z.to_le_bytes(),
            frustum_planes[3].w.to_le_bytes(),
            frustum_planes[4].x.to_le_bytes(),
            frustum_planes[4].y.to_le_bytes(),
            frustum_planes[4].z.to_le_bytes(),
            frustum_planes[4].w.to_le_bytes(),
            frustum_planes[5].x.to_le_bytes(),
            frustum_planes[5].y.to_le_bytes(),
            frustum_planes[5].z.to_le_bytes(),
            frustum_planes[5].w.to_le_bytes(),
        ]
    }

    pub fn calculate_frustum_planes_to_bytes(
        &self,
        view_projection_matrix: Option<Matrix4<f32>>,
        perspective_projection_matrix: Option<Matrix4<f32>>,
    ) -> [[u8; 4]; 6 * 4] {
        let frustum_planes =
            self.calculate_frustum_planes(view_projection_matrix, perspective_projection_matrix);

        Self::frustum_to_bytes(&frustum_planes)
    }

    fn update_buffers(&mut self, queue: &Queue) {
        let view_projection_matrix = self.calculate_view_projection_matrix();
        let perspective_projection_matrix = self.calculate_perspective_projection_matrix();

        let perspective_view_projection_matrix =
            perspective_projection_matrix * view_projection_matrix;

        let view_projection_transposed = view_projection_matrix.transpose();
        let perspective_projection_invert = perspective_projection_matrix
            .invert()
            .unwrap_or(Matrix4::identity());

        queue.write_buffer(
            &self.camera_buffer,
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

        // --- Frustum ---
        self.set_frustum(
            &self.calculate_frustum_planes(
                Some(view_projection_matrix),
                Some(perspective_projection_matrix),
            ),
            queue,
        );
    }

    pub fn set_frustum(&self, frustum_planes: &[Vector4<f32>; 6], queue: &Queue) {
        let data = Self::frustum_to_bytes(frustum_planes);

        self.set_frustum_data(&data, queue);
    }

    pub fn set_frustum_data(&self, frustum_data: &[[u8; 4]; 6 * 4], queue: &Queue) {
        queue.write_buffer(&self.frustum_buffer, 0, &frustum_data.concat());
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

    pub fn camera_bind_group(&self) -> &BindGroup {
        &self.camera_bind_group
    }

    pub fn camera_buffer(&self) -> &Buffer {
        &self.camera_buffer
    }

    pub fn frustum_bind_group(&self) -> &BindGroup {
        &self.frustum_bind_group
    }

    pub fn frustum_buffer(&self) -> &Buffer {
        &self.frustum_buffer
    }
}
