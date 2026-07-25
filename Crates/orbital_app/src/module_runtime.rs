//! Module-based runtime — replaces AppRuntime for ECS-native applications.
//!
//! Uses the `Module` trait instead of `App`. Multiple modules contribute
//! systems to a shared game schedule. The runtime manages core, game, and
//! realize schedules.

use std::sync::{Arc, Mutex};

#[cfg(feature = "gamepad_input")]
use gilrs::Gilrs;
use log::trace;
use orbital_core::logging::{self, debug, error, info, warn};
use orbital_ecs::Schedule;
use orbital_input::{InputEvent, InputState};
use wgpu::CurrentSurfaceTexture;
use wgpu::TextureViewDescriptor;
use winit::{
    application::ApplicationHandler,
    error::EventLoopError,
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{CursorGrabMode, WindowId},
};

use orbital_resources::{
    LightType, ShadowCaster, ShadowLightInfo,
};

use crate::{
    AppContext, AppSettings, AppState, Module, RenderOverlayResource, Timer, make_core_schedule,
};
use orbital_ecs_bridge::{
    CursorGrabConfig, CursorPosition, DeltaTime, DeviceResource, EcsCameraStore, EngineEvent,
    EngineEvents, FrameCounter, InputSnapshot,     LightDescriptorEcs, Position,
    QueueResource, SurfaceFormatResource, TotalTime, WindowSize,
};

macro_rules! ctx_lock {
    ($ctx:ident) => {
        $ctx.lock().expect("Mutex failure")
    };
}

pub struct ModuleRuntime {
    module: Box<dyn Module>,
    settings: AppSettings,
    state: AppState,
    input_state: InputState,
    timer: Option<Timer>,
    ecs_world: orbital_ecs::World,
    core_schedule: Schedule,
    game_schedule: Schedule,
    module_setup_done: bool,
    renderer: Option<orbital_renderer::Renderer>,
    #[cfg(feature = "gamepad_input")]
    gil: Gilrs,
}

impl ModuleRuntime {
    pub fn liftoff<M: Module + 'static>(
        event_loop: EventLoop<()>,
        settings: AppSettings,
        module: M,
    ) -> Result<(), EventLoopError> {
        logging::init();

        info!("Orbital Runtime (Module)");
        info!(" --- @SakulFlee --- ");

        let mut runtime = Self {
            module: Box::new(module),
            settings,
            state: AppState::Starting,
            input_state: InputState::new(),
            timer: None,
            ecs_world: orbital_ecs::World::new(),
            core_schedule: make_core_schedule(),
            game_schedule: Schedule::new(),
            module_setup_done: false,
            renderer: None,
            #[cfg(feature = "gamepad_input")]
            gil: Gilrs::new().expect("Gamepad input initialization failed!"),
        };

        // Initialise built-in ECS resources
        runtime.ecs_world.insert_resource(FrameCounter(0));
        runtime.ecs_world.insert_resource(DeltaTime(0.0));
        runtime.ecs_world.insert_resource(TotalTime(0.0));
        runtime
            .ecs_world
            .insert_resource(InputSnapshot(InputState::new()));
        runtime
            .ecs_world
            .insert_resource(CursorPosition(cgmath::Vector2::new(0.0, 0.0)));
        runtime
            .ecs_world
            .insert_resource(WindowSize(cgmath::Vector2::new(0, 0)));
        runtime.ecs_world.insert_resource(EngineEvents::default());
        runtime.ecs_world.insert_resource(EcsCameraStore::new());

        event_loop.run_app(&mut runtime)
    }

    fn redraw(&mut self) {
        let AppState::Ready(ctx) = &self.state else {
            error!(
                "Trying to redraw when app state is in a non-ready state! ({:?})",
                self.state
            );
            return;
        };

        let lock = ctx_lock!(ctx);

        let frame = match lock.current_surface_texture() {
            CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            CurrentSurfaceTexture::Suboptimal(surface_texture) => {
                warn!("Suboptimal surface texture acquired!");
                surface_texture
            }
            CurrentSurfaceTexture::Timeout => {
                warn!("Acquiring next surface texture encountered a timeout!");
                return;
            }
            CurrentSurfaceTexture::Occluded => {
                warn!("Cannot acquire next surface texture as the surface is current occluded!");
                return;
            }
            CurrentSurfaceTexture::Outdated => {
                warn!("Acquired next surface texture, but its already outdated!");
                return;
            }
            CurrentSurfaceTexture::Lost => {
                warn!("Surface was lost, cannot acquire next surface texture!");
                return;
            }
            CurrentSurfaceTexture::Validation => {
                warn!("Encountered a validation error while acquiring surface texture");
                return;
            }
        };

        let format = lock.get_first_view_format();
        let device = lock.device();
        let queue = lock.queue();

        let view: wgpu::TextureView = frame.texture.create_view(&TextureViewDescriptor {
            format: Some(format),
            ..TextureViewDescriptor::default()
        });

        // Realize all ECS state (needs &mut self.ecs_world)
        crate::systems::realize::realize_cameras(&mut self.ecs_world);
        crate::systems::realize::realize_lights(&mut self.ecs_world);
        crate::systems::realize::realize_environment(&mut self.ecs_world);
        crate::systems::realize::realize_models(&mut self.ecs_world);

        // Freeze/unfreeze the culling frustum (default F4)
        {
            use orbital_input::InputButton;
            use std::sync::atomic::Ordering;
            use winit::keyboard::PhysicalKey;

            static PREV_FREEZE: std::sync::atomic::AtomicBool =
                std::sync::atomic::AtomicBool::new(false);

            let freeze_key = self
                .ecs_world
                .get_resource::<crate::FreezeKeyConfig>()
                .map(|c| c.0)
                .unwrap_or(winit::keyboard::KeyCode::F4);

            let input_res = self.ecs_world.get_resource::<InputSnapshot>();
            let pressed = input_res
                .map(|input| {
                    input
                        .0
                        .button_state_any(&InputButton::Keyboard(PhysicalKey::Code(freeze_key)))
                        .map(|(_, s)| s)
                        .unwrap_or(false)
                })
                .unwrap_or(false);

            if pressed && !PREV_FREEZE.load(Ordering::Relaxed) {
                // Rising edge — toggle frozen state.
                // Read the current state (immutable borrow of ecs_world).
                let was_frozen = self
                    .ecs_world
                    .get_resource::<orbital_ecs_bridge::FrozenFrustum>()
                    .and_then(|f| f.0.clone())
                    .is_some();

                // Read camera data while ecs_world is still immutably borrowed.
                let capture = if !was_frozen {
                    let active_cam = self
                        .ecs_world
                        .get_resource::<orbital_ecs_bridge::ActiveCamera>();
                    let store = self
                        .ecs_world
                        .get_resource::<orbital_ecs_bridge::EcsCameraStore>();
                    active_cam.and_then(|active| {
                        store.and_then(|s| {
                            s.get(active.0.index).map(|arc_cam| {
                                let cam = arc_cam.read().unwrap();
                                orbital_ecs_bridge::FrozenFrustumData {
                                    frustum: cam.frustum(),
                                    perspective_view_projection_matrix: *cam
                                        .perspective_view_projection_matrix(),
                                }
                            })
                        })
                    })
                } else {
                    None
                };

                // Now mutate ecs_world.
                self.ecs_world
                    .insert_resource(orbital_ecs_bridge::FrozenFrustum(if was_frozen {
                        None
                    } else {
                        capture
                    }));
            }
            PREV_FREEZE.store(pressed, Ordering::Relaxed);
        }

        // Frustum culling — dispatches GPU compute to compact visible instances
        // and write indirect draw args.
        crate::systems::cull::sys_frustum_cull(&mut self.ecs_world);

        // Keep a borrow of the cull resource for the render call below.
        // This borrows self.ecs_world immutably — OK because all subsequent
        // accesses of ecs_world are also immutable.
        let cull_res = self
            .ecs_world
            .get_resource::<orbital_ecs_bridge::CullResource>();

        // Extract all rendering data while ecs_world is not mutably borrowed
        let (camera_buffer, light_buffer, env_ibl, model_ptrs, shadow_lights, camera_pvp, camera_near, camera_far) = {
            let cb = self.extract_camera_buffer(device, queue);
            let (pvp, near, far) = self.extract_camera_projection_data();
            let lb = self.extract_light_buffer(device);
            let ei = self.extract_env_ibl();
            let mp = self.collect_model_ptrs();
            let sl = self.collect_shadow_lights();
            (cb, lb, ei, mp, sl, pvp, near, far)
        };

        // IBL BRDF (static cache)
        static BRDF_ONCE: std::sync::OnceLock<orbital_resources::IblBrdf> =
            std::sync::OnceLock::new();
        let brdf = BRDF_ONCE.get_or_init(|| orbital_resources::IblBrdf::generate(device, queue));
        let brdf_tex = brdf.texture_ref();

        // Environment IBL textures (from owned Arc)
        let (env_diff_view, env_diff_sampler, env_spec_view, env_spec_sampler) = match &env_ibl {
            Some(env) => (
                env.ibl_diffuse().view(),
                env.ibl_diffuse().sampler(),
                env.ibl_specular().view(),
                env.ibl_specular().sampler(),
            ),
            None => {
                static FALLBACK_ONCE: std::sync::OnceLock<(
                    orbital_resources::Texture,
                    orbital_resources::Texture,
                )> = std::sync::OnceLock::new();
                let (diff, spec) = FALLBACK_ONCE.get_or_init(|| {
                    (
                        orbital_resources::Texture::create_empty_cube_texture(
                            Some("default IBL diffuse"),
                            cgmath::Vector2::new(1, 1),
                            wgpu::TextureFormat::R8Unorm,
                            wgpu::TextureUsages::TEXTURE_BINDING,
                            1,
                            device,
                        ),
                        orbital_resources::Texture::create_empty_cube_texture(
                            Some("default IBL specular"),
                            cgmath::Vector2::new(1, 1),
                            wgpu::TextureFormat::R8Unorm,
                            wgpu::TextureUsages::TEXTURE_BINDING,
                            1,
                            device,
                        ),
                    )
                });
                (diff.view(), diff.sampler(), spec.view(), spec.sampler())
            }
        };

        // Extract shadow resources from renderer's shadow renderer (with static fallbacks)
        static FALLBACK_SHADOW_BUF: std::sync::OnceLock<wgpu::Buffer> = std::sync::OnceLock::new();
        static FALLBACK_SHADOW_TEX: std::sync::OnceLock<(wgpu::Texture, wgpu::TextureView)> = std::sync::OnceLock::new();
        static FALLBACK_SHADOW_SAMPLER: std::sync::OnceLock<wgpu::Sampler> = std::sync::OnceLock::new();
        static FALLBACK_CUBE_TEX: std::sync::OnceLock<(wgpu::Texture, wgpu::TextureView)> = std::sync::OnceLock::new();
        static FALLBACK_CUBE_SAMPLER: std::sync::OnceLock<wgpu::Sampler> = std::sync::OnceLock::new();

        let shadow_slot_buffer: &wgpu::Buffer;
        let shadow_depth_view: &wgpu::TextureView;
        let shadow_sampler: &wgpu::Sampler;
        let cube_shadow_depth_view: &wgpu::TextureView;
        let cube_shadow_sampler: &wgpu::Sampler;

        if let Some(renderer) = &self.renderer {
            if let Some(sr) = renderer.shadow_renderer() {
                shadow_slot_buffer = sr.slot_data_buffer();
                shadow_depth_view = sr.depth_texture().view();
                shadow_sampler = sr.sampler();
                cube_shadow_depth_view = sr.cube_depth_texture().view();
                cube_shadow_sampler = sr.cube_sampler();
            } else {
                let fb = FALLBACK_SHADOW_BUF.get_or_init(|| {
                    device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Fallback Shadow Buffer"),
                        size: std::mem::size_of::<orbital_resources::ShadowGpuData>() as u64,
                        usage: wgpu::BufferUsages::UNIFORM,
                        mapped_at_creation: false,
                    })
                });
                let (_, fbv) = FALLBACK_SHADOW_TEX.get_or_init(|| {
                    let t = device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("Fallback Shadow Tex"),
                        size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Depth32Float,
                        usage: wgpu::TextureUsages::TEXTURE_BINDING,
                        view_formats: &[],
                    });
                    let v = t.create_view(&Default::default());
                    (t, v)
                });
                let fs = FALLBACK_SHADOW_SAMPLER.get_or_init(|| {
                    device.create_sampler(&wgpu::SamplerDescriptor {
                        label: Some("Fallback Shadow Sampler"),
                        ..Default::default()
                    })
                });
                let (_, cbv) = FALLBACK_CUBE_TEX.get_or_init(|| {
                    let t = device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("Fallback Cube Shadow Tex"),
                        size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 6 },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Depth32Float,
                        usage: wgpu::TextureUsages::TEXTURE_BINDING,
                        view_formats: &[],
                    });
                    let v = t.create_view(&wgpu::TextureViewDescriptor {
                        dimension: Some(wgpu::TextureViewDimension::CubeArray),
                        ..Default::default()
                    });
                    (t, v)
                });
                let cs = FALLBACK_CUBE_SAMPLER.get_or_init(|| {
                    device.create_sampler(&wgpu::SamplerDescriptor {
                        label: Some("Fallback Cube Shadow Sampler"),
                        ..Default::default()
                    })
                });
                shadow_slot_buffer = fb;
                shadow_depth_view = fbv;
                shadow_sampler = fs;
                cube_shadow_depth_view = cbv;
                cube_shadow_sampler = cs;
            }
        } else {
            let fb = FALLBACK_SHADOW_BUF.get_or_init(|| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Fallback Shadow Buffer 2"),
                    size: std::mem::size_of::<orbital_resources::ShadowGpuData>() as u64,
                    usage: wgpu::BufferUsages::UNIFORM,
                    mapped_at_creation: false,
                })
            });
            let (_, fbv) = FALLBACK_SHADOW_TEX.get_or_init(|| {
                let t = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("Fallback Shadow Tex 2"),
                    size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Depth32Float,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                });
                let v = t.create_view(&Default::default());
                (t, v)
            });
            let fs = FALLBACK_SHADOW_SAMPLER.get_or_init(|| {
                device.create_sampler(&wgpu::SamplerDescriptor {
                    label: Some("Fallback Shadow Sampler 2"),
                    ..Default::default()
                })
            });
            let (_, cbv) = FALLBACK_CUBE_TEX.get_or_init(|| {
                let t2 = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("Fallback Cube Shadow Tex 2"),
                    size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 6 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Depth32Float,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                });
                let v2 = t2.create_view(&wgpu::TextureViewDescriptor {
                    dimension: Some(wgpu::TextureViewDimension::CubeArray),
                    ..Default::default()
                });
                (t2, v2)
            });
            let cs = FALLBACK_CUBE_SAMPLER.get_or_init(|| {
                device.create_sampler(&wgpu::SamplerDescriptor {
                    label: Some("Fallback Cube Shadow Sampler 2"),
                    ..Default::default()
                })
            });
            shadow_slot_buffer = fb;
            shadow_depth_view = fbv;
            shadow_sampler = fs;
            cube_shadow_depth_view = cbv;
            cube_shadow_sampler = cs;
        };

        // Build bind group
        let bind_group_layout = orbital_resources::make_world_bind_group_layout(device);
        let world_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("World Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        camera_buffer.as_entire_buffer_binding(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(
                        light_buffer.as_entire_buffer_binding(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(env_diff_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(env_diff_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(env_spec_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(env_spec_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(brdf_tex.view()),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::Sampler(brdf_tex.sampler()),
                },
                // Shadow bindings (optional — use fallback empty resources if no shadow renderer)
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::Buffer(
                        shadow_slot_buffer.as_entire_buffer_binding(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: wgpu::BindingResource::TextureView(shadow_depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 10,
                    resource: wgpu::BindingResource::Sampler(shadow_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 11,
                    resource: wgpu::BindingResource::TextureView(cube_shadow_depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 12,
                    resource: wgpu::BindingResource::Sampler(cube_shadow_sampler),
                },
            ],
        });

        // Collect models (owned Vec of raw pointers)
        let models: Vec<&orbital_resources::Model> = model_ptrs
            .iter()
            .filter_map(|ptr| unsafe { ptr.as_ref() })
            .collect();

        // Render
        if let Some(renderer) = &mut self.renderer {
            let cull = cull_res.as_ref().and_then(|r| r.0.as_ref());
            renderer.render(
                &view,
                &world_bind_group,
                env_ibl.as_ref().map(|a| a.as_ref()),
                models,
                device,
                queue,
                cull,
                &shadow_lights,
                camera_pvp.as_ref(),
                camera_near,
                camera_far,
            );
        }

        // Optional post‑main‑pass overlay (debug viz, HUD, gizmos, …)
        if let Some(overlay_res) = self.ecs_world.get_resource::<RenderOverlayResource>() {
            let camera_buffer = self.extract_camera_buffer(device, queue);
            let ctx = crate::RenderOverlayContext {
                target_view: &view,
                camera_buffer: &camera_buffer,
                device,
                queue,
                ecs: &self.ecs_world,
            };
            overlay_res.0.lock().unwrap().render(ctx);
        }

        lock.queue().present(frame);
    }

    /// Extract camera buffer as an owned Buffer (cheap Arc clone).
    fn extract_camera_buffer(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Buffer {
        let active_entity = self
            .ecs_world
            .get_resource::<orbital_ecs_bridge::ActiveCamera>()
            .map(|a| a.0);

        match active_entity {
            Some(entity) => {
                let store = self
                    .ecs_world
                    .get_resource::<orbital_ecs_bridge::EcsCameraStore>();
                match store {
                    Some(s) => match s.get(entity.index) {
                        Some(arc_camera) => {
                            let guard = arc_camera.read().unwrap();
                            guard.camera_buffer().clone()
                        }
                        None => self.fallback_camera(device, queue),
                    },
                    None => self.fallback_camera(device, queue),
                }
            }
            None => self.fallback_camera(device, queue),
        }
    }

    /// Fallback camera when no ECS camera exists.
    fn fallback_camera(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Buffer {
        static FALLBACK_ONCE: std::sync::OnceLock<wgpu::Buffer> = std::sync::OnceLock::new();
        FALLBACK_ONCE
            .get_or_init(|| {
                let desc = orbital_resources::CameraDescriptor::default();
                let cam = orbital_resources::Camera::from_descriptor(desc, device, queue);
                cam.camera_buffer().clone()
            })
            .clone()
    }

    /// Extract light buffer — returns owned Buffer.
    fn extract_light_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        let light_buf = self
            .ecs_world
            .get_resource::<orbital_ecs_bridge::LightBufferResource>();
        match light_buf {
            Some(r) => match &r.0 {
                Some(buf) => buf.as_ref().clone(),
                None => self.fallback_light(device),
            },
            None => self.fallback_light(device),
        }
    }

    /// Fallback light buffer.
    fn fallback_light(&self, device: &wgpu::Device) -> wgpu::Buffer {
        static FALLBACK_ONCE: std::sync::OnceLock<wgpu::Buffer> = std::sync::OnceLock::new();
        FALLBACK_ONCE
            .get_or_init(|| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Fallback Light Buffer"),
                    size: 64,
                    usage: wgpu::BufferUsages::STORAGE,
                    mapped_at_creation: false,
                })
            })
            .clone()
    }

    /// Extract environment IBL textures.
    /// Returns owned Arc<WorldEnvironment> to keep references valid.
    fn extract_env_ibl(&self) -> Option<Arc<orbital_resources::WorldEnvironment>> {
        self.ecs_world
            .get_resource::<orbital_ecs_bridge::EnvironmentGpuResource>()
            .and_then(|r| r.0.clone())
    }

    /// Collect realized model pointers from ECS.
    /// Returns raw pointers — caller must ensure validity.
    fn collect_model_ptrs(&self) -> Vec<*const orbital_resources::Model> {
        let store = match self
            .ecs_world
            .get_component_store::<orbital_ecs_bridge::ModelRealization>()
        {
            Some(s) => s,
            None => return Vec::new(),
        };
        store
            .dense
            .iter()
            .filter_map(|&eid| {
                store.sparse[eid].map(|idx| {
                    let realization = &store.components[idx];
                    &*realization.0 as *const orbital_resources::Model
                })
            })
            .collect()
    }

    /// Extract camera perspective view-projection matrix, near, and far from the active camera.
    fn extract_camera_projection_data(&self) -> (Option<cgmath::Matrix4<f32>>, f32, f32) {
        let active_entity = self
            .ecs_world
            .get_resource::<orbital_ecs_bridge::ActiveCamera>()
            .map(|a| a.0);

        match active_entity {
            Some(entity) => {
                let store = self
                    .ecs_world
                    .get_resource::<orbital_ecs_bridge::EcsCameraStore>();
                match store {
                    Some(s) => match s.get(entity.index) {
                        Some(arc_camera) => {
                            let guard = arc_camera.read().unwrap();
                            (
                                Some(*guard.perspective_view_projection_matrix()),
                                guard.near(),
                                guard.far(),
                            )
                        }
                        None => (None, 0.1, 1000.0),
                    },
                    None => (None, 0.1, 1000.0),
                }
            }
            None => (None, 0.1, 1000.0),
        }
    }

    /// Collect shadow-casting light info from ECS.
    fn collect_shadow_lights(&self) -> Vec<ShadowLightInfo> {
        let descs = match self
            .ecs_world
            .get_component_store::<LightDescriptorEcs>()
        {
            Some(s) => s,
            None => return Vec::new(),
        };
        let positions = match self
            .ecs_world
            .get_component_store::<Position>()
        {
            Some(s) => s,
            None => return Vec::new(),
        };
        let casters = match self
            .ecs_world
            .get_component_store::<ShadowCaster>()
        {
            Some(s) => s,
            None => return Vec::new(),
        };

        let mut lights = Vec::new();
        let mut light_store_index = 0u32;
        for &eid in descs.dense.as_slice() {
            let desc_idx = match descs.sparse.get(eid).copied().flatten() {
                Some(i) => i,
                None => {
                    light_store_index += 1;
                    continue;
                }
            };
            let desc = &descs.components[desc_idx];

            // Check if this light has a shadow caster
            let has_shadow = match casters.sparse.get(eid).copied().flatten() {
                Some(idx) => casters.components[idx].enabled,
                None => false,
            };

            if has_shadow {
                let pos_idx = match positions.sparse.get(eid).copied().flatten() {
                    Some(i) => i,
                    None => {
                        light_store_index += 1;
                        continue;
                    }
                };
                let caster = &casters.components[casters.sparse[eid].unwrap()];
                let pos = &positions.components[pos_idx];
                let (light_type, outer_cone_angle) = match desc.light_type {
                    LightType::Point { .. } => (0, 0.0),
                    LightType::Directional { .. } => (1, 0.0),
                    LightType::Spot { outer_cone_angle, .. } => (2, outer_cone_angle),
                };
                lights.push(ShadowLightInfo {
                    light_type,
                    direction: desc.direction,
                    position: cgmath::Vector3::new(pos.0.x, pos.0.y, pos.0.z),
                    caster: caster.clone(),
                    outer_cone_angle,
                    light_store_index,
                });
            }
            light_store_index += 1;
        }
        lights
    }

    #[cfg(feature = "gamepad_input")]
    fn receive_controller_inputs(&mut self) {
        while let Some(gil_event) = self.gil.next_event() {
            if let Some(input_event) = InputEvent::convert_gil_event(gil_event) {
                self.input_state.handle_event(input_event);
            }
        }
    }

    fn update(&mut self) -> bool {
        let (delta_time, cycle) = self.timer.as_mut().expect("Timer went missing").tick();

        if let Some((total_delta, fps)) = cycle {
            info!("FPS: {fps} | TDT: {total_delta}s | CDT: {delta_time}s");
        }

        // Write frame-computed engine state into the ECS world
        self.ecs_world.insert_resource(DeltaTime(delta_time));
        self.ecs_world
            .insert_resource(InputSnapshot(self.input_state.clone()));

        // Run core schedule (timing, frame counter)
        self.core_schedule.run(&mut self.ecs_world);

        // Poll importer for completed glTF imports
        crate::systems::sys_poll_importer(&mut self.ecs_world);

        // Run game schedule (user systems)
        self.game_schedule.run(&mut self.ecs_world);

        #[cfg(feature = "gamepad_input_poll")]
        self.receive_controller_inputs();

        // Process engine events
        let exit_requested = self.process_engine_events();

        self.input_state.reset_deltas();

        exit_requested
    }

    fn process_engine_events(&mut self) -> bool {
        let ctx = match &self.state {
            AppState::Ready(ctx) => ctx.clone(),
            _ => {
                debug!(
                    "App in invalid state ({:?}), skipping engine events!",
                    self.state
                );
                return false;
            }
        };

        let events = match self.ecs_world.get_resource_mut::<EngineEvents>() {
            Some(mut e) => e.drain(),
            None => return false,
        };

        let mut exit_requested = false;
        for event in events {
            match event {
                EngineEvent::CursorGrabbed(grab) => {
                    let lock = ctx_lock!(ctx);
                    if grab {
                        if let Err(e) = lock.window().set_cursor_grab(CursorGrabMode::Confined) {
                            error!(
                                "Failed to set cursor grab! This might not be supported on your platform. Error: {e}"
                            );
                        }
                    } else if let Err(e) = lock.window().set_cursor_grab(CursorGrabMode::None) {
                        error!("Failed to unset cursor grab! Error: {e}");
                    }
                }
                EngineEvent::CursorVisible(visible) => {
                    ctx_lock!(ctx).window().set_cursor_visible(visible);
                }
                EngineEvent::RequestClose => {
                    warn!("App closure was requested!");
                    exit_requested = true;
                }
                EngineEvent::ForceClose { exit_code } => {
                    warn!("Force app closure was requested with exit code {exit_code}!");
                    std::process::exit(exit_code);
                }
                EngineEvent::RequestRedraw => {
                    ctx_lock!(ctx).window().request_redraw();
                }
            }
        }

        exit_requested
    }

    fn exit(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.exit();
    }
}

impl ApplicationHandler for ModuleRuntime {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !matches!(self.state, AppState::Starting | AppState::Paused) {
            debug!(
                "Attempting to resume while not in required state! (State: {:?})",
                self.state
            );
        }

        debug!("Resuming app ...");

        let ctx = match AppContext::new(event_loop, &self.settings) {
            Ok(ctx) => ctx,
            Err(e) => {
                error!("Critical error: Failed to acquire context while resuming app!");
                trace!("Error: {:?}", e);
                return;
            }
        };

        let config = ctx.make_surface_configuration(self.settings.vsync_enabled);

        self.state = AppState::Ready(Arc::new(Mutex::new(ctx)));

        self.timer = Some(Timer::new());

        if let AppState::Ready(ctx_arc) = &self.state {
            let ctx_guard = ctx_lock!(ctx_arc);

            self.ecs_world
                .insert_resource(DeviceResource(Arc::new(ctx_guard.device().clone())));
            self.ecs_world
                .insert_resource(QueueResource(Arc::new(ctx_guard.queue().clone())));
            self.ecs_world
                .insert_resource(SurfaceFormatResource(config.format));

            // Initialize import pipeline resources
            self.ecs_world
                .insert_resource(orbital_ecs_bridge::ImportQueueResource::default());
            self.ecs_world
                .insert_resource(orbital_ecs_bridge::ImporterResource::new(4));
            self.ecs_world
                .insert_resource(orbital_ecs_bridge::MeshCacheResource::default());
            self.ecs_world
                .insert_resource(orbital_ecs_bridge::MaterialCacheResource::default());

            // Create renderer
            self.renderer = Some(orbital_renderer::Renderer::new(
                config.format,
                cgmath::Vector2::new(config.width, config.height),
                ctx_guard.device(),
                ctx_guard.queue(),
            ));

            // Enable shadows with default settings
            if let Some(renderer) = &mut self.renderer {
                renderer.enable_shadows(
                    ctx_guard.device(),
                    ctx_guard.queue(),
                    16,    // max shadow slots
                    2048,  // shadow map resolution
                );
            }

            // Call Module::setup() and build game schedule
            if !self.module_setup_done {
                let systems =
                    self.module
                        .setup(&mut self.ecs_world, ctx_guard.device(), ctx_guard.queue());
                for system in systems {
                    self.game_schedule.add_system_boxed(system);
                }

                // Register engine-level systems
                // (sys_poll_importer is called directly in update() since it takes &mut World)

                self.module_setup_done = true;
                info!(
                    "Module setup complete, game schedule has {} systems",
                    self.game_schedule.system_count()
                );
            }

            // Auto-grab cursor if configured
            if let Some(config) = self.ecs_world.get_resource::<CursorGrabConfig>()
                && config.0
            {
                if let Err(e) = ctx_guard
                    .window()
                    .set_cursor_grab(winit::window::CursorGrabMode::Confined)
                {
                    log::error!("Failed to grab cursor: {e}");
                }
                ctx_guard.window().set_cursor_visible(false);
            }
        }

        info!("App resumed.");
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        if !matches!(self.state, AppState::Ready { .. }) {
            debug!(
                "Attempting to suspend while not in ready state! (State: {:?})",
                self.state
            );
        }

        self.state = AppState::Paused;
        info!("App suspended!");
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let ctx = match &self.state {
            AppState::Ready(ctx) => ctx.clone(),
            _ => {
                debug!(
                    "App in invalid state ({:?}), skipping window events!",
                    self.state
                );
                return;
            }
        };

        let input_event = match event {
            WindowEvent::CloseRequested => {
                info!("App shutdown requested!");
                self.exit(event_loop);
                return;
            }
            WindowEvent::RedrawRequested => {
                if self.update() {
                    info!("App shutdown requested!");
                    self.exit(event_loop);
                    return;
                }
                self.redraw();

                #[cfg(feature = "auto_request_redraw")]
                ctx_lock!(ctx).window().request_redraw();

                None
            }
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => Some(InputEvent::KeyboardButton {
                device_id,
                event,
                is_synthetic,
            }),
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => Some(InputEvent::MouseButton {
                device_id,
                state,
                button,
            }),
            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
            } => Some(InputEvent::MouseWheel {
                device_id,
                delta,
                phase,
            }),
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                if let Some(mut pos) = self.ecs_world.get_resource_mut::<CursorPosition>() {
                    pos.0 = cgmath::Vector2::new(position.x, position.y);
                }
                Some(InputEvent::MouseMovedPosition {
                    device_id,
                    position,
                })
            }
            WindowEvent::Resized(new_size) => {
                let ctx_lock = ctx_lock!(ctx);
                let configuration =
                    ctx_lock.make_surface_configuration(self.settings.vsync_enabled);
                ctx_lock.reconfigure_surface(&configuration);

                self.input_state.surface_resize(new_size);
                self.ecs_world
                    .insert_resource(WindowSize(cgmath::Vector2::new(
                        new_size.width,
                        new_size.height,
                    )));

                None
            }
            _ => None,
        };

        if let Some(input_event) = input_event {
            self.input_state.handle_event(input_event);
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        device_event: DeviceEvent,
    ) {
        if !matches!(&self.state, AppState::Ready(_)) {
            debug!(
                "App in invalid state ({:?}), skipping device events!",
                self.state
            );
            return;
        }

        if let Some(input_event) = InputEvent::convert_device_event(device_id, device_event) {
            self.input_state.handle_event(input_event);
        }
    }
}
