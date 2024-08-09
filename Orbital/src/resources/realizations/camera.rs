use cgmath::{perspective, Deg, InnerSpace, Matrix4, Vector3};
use std::mem;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsages,
    Device, Queue, ShaderStages,
};

use crate::{game::CameraChange, resources::descriptors::CameraDescriptor};

#[derive(Debug)]
pub struct Camera {
    descriptor: CameraDescriptor,
    bind_group: BindGroup,
    buffer: Buffer,
}

impl Camera {
    pub fn bind_group_layout_descriptor() -> BindGroupLayoutDescriptor<'static> {
        BindGroupLayoutDescriptor {
            label: Some("Camera"),
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
        }
    }

    pub fn from_descriptor(descriptor: CameraDescriptor, device: &Device, queue: &Queue) -> Self {
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Camera Buffer"),
            size: (
                // Main data type is f32.
                // The matrix is a 4x4 f32 vector.
                // Additionally, we have a position 3x f32 vector.
                // Unfortunately, we need to waste one byte here as WGPU
                // wants it to align to the next multiple that is dividable by 8
                // Thus, we need to add one byte here and fill an empty byte at
                // the end.
                mem::size_of::<f32>() * ((4 * 4) + 4)
            ) as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group_layout =
            device.create_bind_group_layout(&Self::bind_group_layout_descriptor());
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

        queue.write_buffer(
            &self.buffer,
            0,
            &[
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
                self.descriptor.position.x.to_le_bytes(),
                self.descriptor.position.y.to_le_bytes(),
                self.descriptor.position.z.to_le_bytes(),
                [0u8; 4], // Empty to align with "dividable by 8"
            ]
            .concat(),
        );
    }

    pub fn calculate_view_projection_matrix(&self) -> Matrix4<f32> {
        // WGPU uses the same coordinate system as found in e.g. DirectX or
        // Metal. Meaning, that the clipping zone is expected to be between
        // -1.0 and +1.0 for the X and Y axis, but 0.0 to +1.0 for the Z axis.
        //
        // However, most computer graphics related library expect OpenGL's
        // coordinate system. OpenGL uses the same X and Y axis normalized
        // space, but puts the Z axis **also from -1.0** to +1.0.
        //
        // This isn't needed! But an object at origin (0.0, 0.0, 0.0) would be
        // halfway in the clipping zone. Using this matrix converts FROM the
        // OpenGL system (as produced by cgmath) INTO the WGPU/DirectX/Metal
        // system.
        #[rustfmt::skip]
        const OPEN_GL_MATRIX: Matrix4<f32> = Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.5,
            0.0, 0.0, 0.0, 1.0,
        );

        // Takes yaw and pitch values and converts them into a target vector for our camera.
        let (pitch_sin, pitch_cos) = self.descriptor.pitch.sin_cos();
        let (yaw_sin, yaw_cos) = self.descriptor.yaw.sin_cos();

        // Calculates the view project matrix
        let view_projection_matrix: Matrix4<f32> = Matrix4::look_to_rh(
            self.descriptor.position,
            Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize(),
            Vector3::unit_y(),
        );

        // Calculates the perspective matrix
        let perspective_matrix = perspective(
            Deg(self.descriptor.fovy),
            self.descriptor.aspect,
            self.descriptor.near,
            self.descriptor.far,
        );

        // Final result :)
        OPEN_GL_MATRIX * perspective_matrix * view_projection_matrix
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
