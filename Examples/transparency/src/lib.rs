use std::sync::Arc;

use orbital::app::{App, AppSettings, Module, sys_camera_controller};
use orbital::cgmath::{InnerSpace, Point3, Quaternion, Rad, Vector2, Vector3};
use orbital::ecs::{IntoSystem, System, World};
use orbital::ecs_bridge::{
    ActiveCamera, CameraDescriptor, CursorGrabConfig, EnvironmentDescriptorResource,
    LightDescriptor, LightDirty, ModelDescriptor, ModelDirty, ModelInstances, Position, Rotation,
};
use orbital::logging::{self, error, info};
use orbital::procgeo::shapes::{box_, plane};
use orbital::resources::{
    AlphaMode, FilterMode, GeneratedSkyParameters, MaterialShaderDescriptor, MeshDescriptor,
    PBRMaterialShaderDescriptor, SamplingType, ShadowCaster, TextureDescriptor, TextureSize,
    Transform, WorldEnvironmentDescriptor,
};
use orbital::wgpu::{
    BlendComponent, BlendFactor, BlendOperation, BlendState, TextureDimension, TextureFormat,
    TextureUsages, TextureViewDimension,
};

pub const NAME: &str = env!("CARGO_PKG_NAME");

#[orbital::main]
pub fn entrypoint(
    event_loop_result: Result<
        orbital::winit::event_loop::EventLoop<()>,
        orbital::winit::error::EventLoopError,
    >,
) {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    logging::init();

    let event_loop = event_loop_result.expect("Event Loop failure");

    let app_settings = AppSettings {
        vsync_enabled: true,
        name: NAME.to_string(),
        back_presses_to_exit: 3,
        ..AppSettings::default()
    };

    match App::new()
        .add_module(GameModule)
        .add_module(orbital::debug_ui::DebugUiModule::new())
        .add_module(orbital::touch_ui::TouchUiModule)
        .liftoff(event_loop, app_settings)
    {
        Ok(()) => info!("Cleanly exited!"),
        Err(e) => error!("Runtime failure: {e:?}"),
    }
}

struct GameModule;

impl Module for GameModule {
    fn setup(
        &self,
        ecs: &mut World,
        _device: &orbital::wgpu::Device,
        _queue: &orbital::wgpu::Queue,
    ) -> Vec<Box<dyn System>> {
        let camera = ecs.spawn_entity();
        ecs.attach_component(
            &camera,
            CameraDescriptor {
                label: "Default".into(),
                aspect: 16.0 / 9.0,
                fovy: Rad(std::f32::consts::FRAC_PI_4),
                projection_mode: Default::default(),
                ortho_height: 10.0,
                near: 0.1,
                far: 10000.0,
                global_gamma: 2.2,
            },
        )
        .unwrap();
        ecs.attach_component(&camera, Position(Point3::new(0.0, 3.0, 5.0)))
            .unwrap();
        let mut rot = Rotation::identity();
        rot.rotate_yaw(Rad(std::f32::consts::FRAC_PI_2));
        rot.rotate_pitch(Rad(-0.3));
        ecs.attach_component(&camera, rot).unwrap();
        ecs.insert_resource(ActiveCamera(camera));
        ecs.insert_resource(CursorGrabConfig(true));

        ecs.insert_resource(EnvironmentDescriptorResource(Some(
            WorldEnvironmentDescriptor::Generated {
                cube_face_size: 256,
                sampling_type: SamplingType::GaussianBlur,
                custom_specular_mip_level_count: Some(3),
                parameters: Some(GeneratedSkyParameters::default()),
                dynamic: true,
            },
        )));

        spawn_model(
            ecs,
            "floor",
            plane(Vector2::new(10.0, 10.0), 2),
            opaque_material([0.12, 0.13, 0.16, 1.0], 0.0, 0.95),
            Point3::new(0.0, 0.0, 0.0),
        );
        spawn_model(
            ecs,
            "opaque-reference",
            box_(Vector3::new(1.4, 1.4, 1.4)),
            opaque_material([0.75, 0.2, 0.12, 1.0], 0.0, 0.55),
            Point3::new(-2.2, 0.7, 0.0),
        );
        spawn_model(
            ecs,
            "opaque-backdrop",
            box_(Vector3::new(8.0, 4.0, 0.2)),
            opaque_material([0.08, 0.1, 0.16, 1.0], 0.0, 0.9),
            Point3::new(0.0, 2.0, -2.2),
        );

        // Three overlapping blended panes. The renderer sorts these
        // back-to-front by view distance.
        let panes = [
            ("red-pane", [0.9, 0.12, 0.1, 0.35], 0.8),
            ("green-pane", [0.1, 0.8, 0.2, 0.35], 0.2),
            ("blue-pane", [0.1, 0.3, 0.95, 0.35], -0.4),
        ];
        for (label, color, z) in panes {
            spawn_model(
                ecs,
                label,
                box_(Vector3::new(2.2, 2.0, 0.06)),
                blend_material(color),
                Point3::new(0.0, 1.2, z),
            );
        }

        // Additive and multiplicative blend states use the same forward
        // transparent queue and back-to-front sort.
        spawn_model(
            ecs,
            "additive-pane",
            box_(Vector3::new(2.2, 2.0, 0.06)),
            custom_blend_material(
                [0.04, 0.08, 0.2, 1.0],
                BlendState {
                    color: BlendComponent {
                        src_factor: BlendFactor::One,
                        dst_factor: BlendFactor::One,
                        operation: BlendOperation::Add,
                    },
                    alpha: BlendComponent {
                        src_factor: BlendFactor::One,
                        dst_factor: BlendFactor::One,
                        operation: BlendOperation::Add,
                    },
                },
            ),
            Point3::new(0.0, 1.2, -0.9),
        );
        spawn_model(
            ecs,
            "multiply-pane",
            box_(Vector3::new(2.2, 2.0, 0.06)),
            custom_blend_material(
                [0.65, 0.8, 1.0, 1.0],
                BlendState {
                    color: BlendComponent {
                        src_factor: BlendFactor::Dst,
                        dst_factor: BlendFactor::Zero,
                        operation: BlendOperation::Add,
                    },
                    alpha: BlendComponent {
                        src_factor: BlendFactor::Zero,
                        dst_factor: BlendFactor::One,
                        operation: BlendOperation::Add,
                    },
                },
            ),
            Point3::new(0.0, 1.2, -1.5),
        );

        // Alpha-masked geometry: the checker's transparent cells are
        // discarded in the shadow pass, producing a perforated shadow.
        spawn_model(
            ecs,
            "masked-caster",
            box_(Vector3::new(1.6, 1.6, 0.08)),
            mask_material(checker_alpha()),
            Point3::new(2.2, 1.4, 0.0),
        );

        let spot_pos = Point3::new(3.0, 5.0, 3.0);
        let target = Point3::new(0.0, 0.5, 0.0);
        let dir = (target - spot_pos).normalize();
        let spot = ecs.spawn_entity();
        ecs.attach_component(
            &spot,
            LightDescriptor::new_spot(
                Vector3::new(1.0, 1.0, 1.0),
                45.0,
                Vector3::new(dir.x, dir.y, dir.z),
                0.1,
                0.44,
            ),
        )
        .unwrap();
        ecs.attach_component(&spot, Position(spot_pos)).unwrap();
        ecs.attach_component(&spot, LightDirty(true)).unwrap();
        ecs.attach_component(
            &spot,
            ShadowCaster {
                cascade_count: 0,
                bias: 0.0002,
                ..Default::default()
            },
        )
        .unwrap();

        let fill = ecs.spawn_entity();
        ecs.attach_component(
            &fill,
            LightDescriptor::new_directional(
                Vector3::new(0.0, -0.8, -0.6),
                Vector3::new(1.0, 0.92, 0.8),
                0.3,
            ),
        )
        .unwrap();
        ecs.attach_component(&fill, Position(Point3::new(0.0, 0.0, 0.0)))
            .unwrap();
        ecs.attach_component(&fill, LightDirty(true)).unwrap();

        vec![sys_camera_controller.into_system()]
    }
}

fn spawn_model(
    ecs: &mut World,
    label: &str,
    mesh: MeshDescriptor,
    material: Arc<MaterialShaderDescriptor>,
    position: Point3<f32>,
) {
    let entity = ecs.spawn_entity();
    ecs.attach_component(
        &entity,
        ModelDescriptor {
            label: label.into(),
            mesh: Arc::new(mesh),
            materials: vec![material],
        },
    )
    .unwrap();
    let mut instances = ModelInstances::new();
    instances.add_instance(Transform::new(
        Vector3::new(position.x, position.y, position.z),
        Quaternion::new(1.0, 0.0, 0.0, 0.0),
        Vector3::new(1.0, 1.0, 1.0),
    ));
    ecs.attach_component(&entity, instances).unwrap();
    ecs.attach_component(&entity, ModelDirty(true)).unwrap();
}

fn opaque_material(
    albedo: [f32; 4],
    metallic: f32,
    roughness: f32,
) -> Arc<MaterialShaderDescriptor> {
    pbr_material(
        color_texture(albedo),
        metallic,
        roughness,
        AlphaMode::Opaque,
    )
}

fn blend_material(albedo: [f32; 4]) -> Arc<MaterialShaderDescriptor> {
    pbr_material(color_texture(albedo), 0.0, 0.35, AlphaMode::Blend)
}

fn mask_material(albedo: TextureDescriptor) -> Arc<MaterialShaderDescriptor> {
    pbr_material(albedo, 0.0, 0.65, AlphaMode::mask(0.5))
}

fn pbr_material(
    albedo: TextureDescriptor,
    metallic: f32,
    roughness: f32,
    alpha_mode: AlphaMode,
) -> Arc<MaterialShaderDescriptor> {
    Arc::new(pbr_descriptor(albedo, metallic, roughness, alpha_mode))
}

fn custom_blend_material(albedo: [f32; 4], blend: BlendState) -> Arc<MaterialShaderDescriptor> {
    let mut descriptor = pbr_descriptor(color_texture(albedo), 0.0, 0.35, AlphaMode::Blend);
    descriptor.blend = blend;
    Arc::new(descriptor)
}

fn pbr_descriptor(
    albedo: TextureDescriptor,
    metallic: f32,
    roughness: f32,
    alpha_mode: AlphaMode,
) -> MaterialShaderDescriptor {
    PBRMaterialShaderDescriptor {
        albedo,
        metallic: one_channel(metallic),
        metallic_factor: 1.0,
        roughness: one_channel(roughness),
        roughness_factor: 1.0,
        normal: flat_normal(),
        occlusion: one_channel(1.0),
        emissive: black_emissive(),
        alpha_mode,
        alpha_factor: 1.0,
        ..PBRMaterialShaderDescriptor::default()
    }
    .into()
}

fn flat_normal() -> TextureDescriptor {
    TextureDescriptor::Data {
        pixels: vec![128, 128, 255, 255],
        size: TextureSize {
            width: 1,
            height: 1,
            ..Default::default()
        },
        usages: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        format: TextureFormat::Rgba8Unorm,
        texture_dimension: TextureDimension::D2,
        texture_view_dimension: TextureViewDimension::D2,
        filter_mode: FilterMode::linear(),
    }
}

fn black_emissive() -> TextureDescriptor {
    TextureDescriptor::Data {
        pixels: vec![0, 0, 0, 255],
        size: TextureSize {
            width: 1,
            height: 1,
            ..Default::default()
        },
        usages: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        format: TextureFormat::Rgba8Unorm,
        texture_dimension: TextureDimension::D2,
        texture_view_dimension: TextureViewDimension::D2,
        filter_mode: FilterMode::linear(),
    }
}

fn color_texture(color: [f32; 4]) -> TextureDescriptor {
    let pixels = color
        .iter()
        .map(|channel| (channel.clamp(0.0, 1.0) * 255.0) as u8)
        .collect();

    TextureDescriptor::Data {
        pixels,
        size: TextureSize {
            width: 1,
            height: 1,
            ..Default::default()
        },
        usages: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        format: TextureFormat::Rgba8UnormSrgb,
        texture_dimension: TextureDimension::D2,
        texture_view_dimension: TextureViewDimension::D2,
        filter_mode: FilterMode::linear(),
    }
}

fn one_channel(value: f32) -> TextureDescriptor {
    TextureDescriptor::Data {
        pixels: vec![(value.clamp(0.0, 1.0) * 255.0) as u8],
        size: TextureSize {
            width: 1,
            height: 1,
            ..Default::default()
        },
        usages: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        format: TextureFormat::R8Unorm,
        texture_dimension: TextureDimension::D2,
        texture_view_dimension: TextureViewDimension::D2,
        filter_mode: FilterMode::linear(),
    }
}

fn checker_alpha() -> TextureDescriptor {
    const SIZE: u32 = 64;
    let mut pixels = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let visible = ((x / 8) + (y / 8)) % 2 == 0;
            pixels.extend_from_slice(if visible {
                &[255, 255, 255, 255]
            } else {
                &[255, 255, 255, 0]
            });
        }
    }

    TextureDescriptor::Data {
        pixels,
        size: TextureSize {
            width: SIZE,
            height: SIZE,
            ..Default::default()
        },
        usages: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        format: TextureFormat::Rgba8UnormSrgb,
        texture_dimension: TextureDimension::D2,
        texture_view_dimension: TextureViewDimension::D2,
        filter_mode: FilterMode::linear(),
    }
}
