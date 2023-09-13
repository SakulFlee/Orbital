use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use cgmath::{perspective, Deg, InnerSpace, Matrix4, Point3, SquareMatrix, Vector3, Zero};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, Device, Queue,
    ShaderStages,
};
use winit::event::VirtualKeyCode;

use crate::{AppInputHandler, AppObject};

pub struct Camera {
    device: Arc<Device>,
    queue: Arc<Queue>,
    eye: Point3<f32>,
    target: Point3<f32>,
    up: Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
    speed: Vector3<f32>,
    view_projection: Matrix4<f32>,
    uniform: CameraUniform,
    buffer: Option<Buffer>,
    bind_group_layout: Option<BindGroupLayout>,
    bind_group: Option<BindGroup>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct CameraUniform {
    // Note: cgmath and bytemuck are incompatible, thus an abstraction.
    view_proj: [[f32; 4]; 4],
}

impl Camera {
    #[rustfmt::skip]
    pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.5,
        0.0, 0.0, 0.0, 1.0,
    );

    pub const MAX_SPEED: f32 = 1.0;
    pub const ACCELERATION_SPEED: f32 = 0.1;
    pub const DECELERATION_SPEED: f32 = 0.05;

    pub fn new(device: Arc<Device>, queue: Arc<Queue>) -> Self {
        let mut s = Self {
            device,
            queue,
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: Vector3::unit_y(),
            // Width / Height = Aspect :: MIGHT need to be adjusted on window resize
            // 1920 / 1080 = 1.7...
            // TODO
            aspect: 1.7,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
            speed: Vector3::zero(),
            view_projection: Matrix4::identity(),
            uniform: CameraUniform {
                view_proj: Matrix4::identity().into(),
            },
            buffer: None,
            bind_group_layout: None,
            bind_group: None,
        };

        // First update the view projection matrix
        s.update_view_projection_matrix();
        s.uniform.view_proj = s.view_projection.into();

        s.buffer = Some(s.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[s.uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        }));

        s.bind_group_layout = Some(
            s.device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
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
                }),
        );

        s.bind_group = Some(s.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &s.get_bind_group_layout(),
            entries: &[BindGroupEntry {
                binding: 0,
                resource: s.get_buffer().as_entire_binding(),
            }],
        }));

        // After the other structures are filled, call a full update:
        s.update_view_projection_matrix_uniform();

        return s;
    }

    pub fn update_view_projection_matrix(&mut self) {
        let view_matrix = Matrix4::look_at_rh(self.eye, self.target, self.up);
        let projection_matrix = perspective(Deg(self.fovy), self.aspect, self.znear, self.zfar);

        self.view_projection = Camera::OPENGL_TO_WGPU_MATRIX * projection_matrix * view_matrix;
    }

    pub fn update_view_projection_matrix_uniform(&mut self) {
        self.update_view_projection_matrix();
        self.uniform.view_proj = self.view_projection.into();

        self.queue
            .write_buffer(self.get_buffer(), 0, bytemuck::cast_slice(&[self.uniform]));
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

    pub fn get_buffer(&self) -> &Buffer {
        self.buffer
            .as_ref()
            .expect("Camera::get_buffer called, but no resource found!")
    }

    pub fn get_bind_group_layout(&self) -> &BindGroupLayout {
        self.bind_group_layout
            .as_ref()
            .expect("Camera::get_bind_group_layout called, but no resource found!")
    }

    pub fn get_bind_group(&self) -> &BindGroup {
        self.bind_group
            .as_ref()
            .expect("Camera::get_bind_group called, but no resource found!")
    }

    pub fn get_view_projection(&self) -> &Matrix4<f32> {
        &self.view_projection
    }

    pub fn get_view_project_uniform(&self) -> &CameraUniform {
        &self.uniform
    }
}

impl AppObject for Camera {
    fn on_dynamic_update(&mut self, delta_time: f64) {
        // Movement
        self.eye += self.speed * delta_time as f32;
        self.target += self.speed * delta_time as f32;

        let decelerator = Camera::DECELERATION_SPEED * delta_time as f32;
        if self.speed.x > 0.1 && self.speed.x < -0.1 {
            self.speed.x -= decelerator;
        }
        if self.speed.y > 0.1 && self.speed.y < -0.1 {
            self.speed.y -= decelerator;
        }
        if self.speed.z > 0.1 && self.speed.z < -0.1 {
            self.speed.z -= decelerator;
        }

        if self.speed.x > 0.0 {
            if self.speed.x < 0.1 {
                self.speed.x = 0.0;
            }
        } else if self.speed.x < 0.0 {
            if self.speed.x > -0.1 {
                self.speed.x = 0.0;
            }
        }
        if self.speed.y > 0.0 {
            if self.speed.y < 0.1 {
                self.speed.y = 0.0;
            }
        } else if self.speed.y < 0.0 {
            if self.speed.y > -0.1 {
                self.speed.y = 0.0;
            }
        }
        if self.speed.z > 0.0 {
            if self.speed.z < 0.1 {
                self.speed.z = 0.0;
            }
        } else if self.speed.z < 0.0 {
            if self.speed.z > -0.1 {
                self.speed.z = 0.0;
            }
        }

        self.update_view_projection_matrix_uniform();
    }
    fn do_dynamic_update(&self) -> bool {
        true
    }

    fn on_input(&mut self, delta_time: f64, input_handler: &AppInputHandler) {
        let go_forward = input_handler.is_key_pressed(&VirtualKeyCode::W);
        let go_backward = input_handler.is_key_pressed(&VirtualKeyCode::S);
        let go_left = input_handler.is_key_pressed(&VirtualKeyCode::A);
        let go_right = input_handler.is_key_pressed(&VirtualKeyCode::D);

        if !(go_forward || go_backward || go_left || go_right) {
            return;
        }

        let forward = (self.target - self.eye).normalize();
        let backward = -forward;
        let right = forward.cross(self.up).normalize();
        let left = -right;

        if go_forward {
            self.speed += forward * Camera::ACCELERATION_SPEED * delta_time as f32;
        }
        if go_backward {
            self.speed += backward * Camera::ACCELERATION_SPEED * delta_time as f32;
        }
        if go_left {
            self.speed += left * Camera::ACCELERATION_SPEED * delta_time as f32;
        }
        if go_right {
            self.speed += right * Camera::ACCELERATION_SPEED * delta_time as f32;
        }

        if self.speed.x > 1.0 {
            self.speed.x = 1.0;
        } else if self.speed.x < -1.0 {
            self.speed.x = -1.0;
        }
        if self.speed.y > 1.0 {
            self.speed.y = 1.0;
        } else if self.speed.y < -1.0 {
            self.speed.y = -1.0;
        }
        if self.speed.z > 1.0 {
            self.speed.z = 1.0;
        } else if self.speed.z < -1.0 {
            self.speed.z = -1.0;
        }

        log::debug!("Speed: {:?}", self.speed);
    }
    fn do_input(&self) -> bool {
        true
    }
}
