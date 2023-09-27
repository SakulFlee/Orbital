use cgmath::{perspective, Deg, Matrix4, Point3, SquareMatrix, Vector3};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, Device, Queue,
    ShaderStages,
};

mod camera_uniform;
pub use camera_uniform::*;

mod camera_change;
pub use camera_change::*;

pub struct Camera {
    eye: Point3<f32>,
    target: Point3<f32>,
    up: Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
    view_projection: Matrix4<f32>,
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

    pub const BIND_GROUP_LAYOUT_DESCRIPTOR: BindGroupLayoutDescriptor<'static> =
        BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        };

    pub fn from_window_size(device: &Device, queue: &Queue, window_size: (u32, u32)) -> Self {
        Self::new(
            device,
            queue,
            Self::DEFAULT_CAMERA_EYE_POSITION.into(),
            (0.0, 0.0, 0.0).into(),
            Vector3::unit_y(),
            window_size.0 as f32 / window_size.1 as f32,
            45.0,
            0.1,
            100.0,
        )
    }

    #[allow(clippy::too_many_arguments)] // TODO: Check clippy again after Logical/Physical Device split
    pub fn new(
        device: &Device,
        queue: &Queue,
        eye: Point3<f32>,
        target: Point3<f32>,
        up: Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        let empty_uniform = CameraUniform::empty();
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[empty_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group_layout = Self::get_bind_group_layout(device);

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        let mut camera = Self {
            eye,
            target,
            up,
            aspect,
            fovy,
            znear,
            zfar,
            view_projection: Matrix4::identity(),
            buffer,
            bind_group,
        };

        camera.update_buffer(queue);

        camera
    }

    fn update_view_projection_matrix(&mut self) {
        let view_matrix = Matrix4::look_at_rh(self.eye, self.target, self.up);
        let projection_matrix = perspective(Deg(self.fovy), self.aspect, self.znear, self.zfar);

        self.view_projection = Camera::OPENGL_TO_WGPU_MATRIX * projection_matrix * view_matrix;
    }

    pub fn update_buffer(&mut self, queue: &Queue) {
        // Make sure the view matrix is up-to-date
        self.update_view_projection_matrix();

        // Transform into uniform
        let uniform = self.to_uniform();

        // Write uniform into buffer
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[uniform]))
    }

    pub fn to_uniform(&self) -> CameraUniform {
        CameraUniform::from_camera(self)
    }

    pub fn apply_camera_change(&mut self, queue: &Queue, camera_change: CameraChange) {
        if let Some(eye) = camera_change.get_eye() {
            self.set_eye(eye);
        }

        if let Some(target) = camera_change.get_target() {
            self.set_target(target);
        }

        if let Some(up) = camera_change.get_up() {
            self.set_up(up);
        }

        if let Some(aspect) = camera_change.get_aspect() {
            self.set_aspect(aspect);
        }

        if let Some(fovy) = camera_change.get_fovy() {
            self.set_fovy(fovy);
        }

        if let Some(znear) = camera_change.get_znear() {
            self.set_znear(znear);
        }

        if let Some(zfar) = camera_change.get_zfar() {
            self.set_zfar(zfar);
        }

        self.update_buffer(queue);
    }

    pub fn get_eye(&self) -> Point3<f32> {
        self.eye
    }

    pub fn set_eye(&mut self, eye: Point3<f32>) {
        self.eye = eye;
    }

    pub fn get_target(&self) -> Point3<f32> {
        self.target
    }

    pub fn set_target(&mut self, target: Point3<f32>) {
        self.target = target;
    }

    pub fn get_up(&self) -> Vector3<f32> {
        self.up
    }

    pub fn set_up(&mut self, up: Vector3<f32>) {
        self.up = up;
    }

    pub fn get_aspect(&self) -> f32 {
        self.aspect
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }

    pub fn get_fovy(&self) -> f32 {
        self.fovy
    }

    pub fn set_fovy(&mut self, fovy: f32) {
        self.fovy = fovy;
    }

    pub fn get_znear(&self) -> f32 {
        self.znear
    }

    pub fn set_znear(&mut self, znear: f32) {
        self.znear = znear;
    }

    pub fn get_zfar(&self) -> f32 {
        self.zfar
    }

    pub fn set_zfar(&mut self, zfar: f32) {
        self.zfar = zfar;
    }

    pub fn get_view_projection(&self) -> Matrix4<f32> {
        self.view_projection
    }

    pub fn get_buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn get_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&Self::BIND_GROUP_LAYOUT_DESCRIPTOR)
    }

    pub fn get_bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}
