use cgmath::{perspective, Deg, EuclideanSpace, InnerSpace, Matrix4, Point3, Vector3};
use std::mem;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsages,
    Device, Queue, ShaderStages,
};

use crate::resources::CameraDescriptor;

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
                // Main data type
                mem::size_of::<f32>() *
                // View Matrix (4x4 f32)
                (4 * 4)
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
            descriptor: descriptor.clone(),
            // perspective: Perspective3::new(
            //     descriptor.aspect,
            //     descriptor.fovy,
            //     descriptor.znear,
            //     descriptor.zfar,
            // ),
            bind_group,
            buffer,
        };
        camera.update_buffer(queue);
        camera
    }

    pub fn update_from_descriptor(
        &mut self,
        descriptor: CameraDescriptor,
        _device: &Device,
        queue: &Queue,
    ) {
        // TODO: OpenGL to WebGPU matrix?
        self.descriptor = descriptor;
        // self.perspective = Perspective3::new(
        //     descriptor.aspect,
        //     descriptor.fovy,
        //     descriptor.znear,
        //     descriptor.zfar,
        // );

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
            ]
            .concat(),
        );
    }

    pub fn calculate_view_projection_matrix(&self) -> Matrix4<f32> {
        let (pitch_sin, pitch_cos) = self.descriptor.pitch.sin_cos();
        let (yaw_sin, yaw_cos) = self.descriptor.yaw.sin_cos();

        #[rustfmt::skip]
        const OPEN_GL_MATRIX: Matrix4<f32> = Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.5,
            0.0, 0.0, 0.0, 1.0,
        );

        let view_projection_matrix = Matrix4::look_at_rh(
            self.descriptor.position,
            Point3::from_vec(
                Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize(),
            ),
            Vector3::unit_y(),
        );

        let perspective_matrix = perspective(
            Deg(self.descriptor.fovy),
            self.descriptor.aspect,
            self.descriptor.near,
            self.descriptor.far,
        );

        return OPEN_GL_MATRIX * perspective_matrix * view_projection_matrix;
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
