use orbital::app::{App, AppSettings, Module, sys_camera_controller};
use orbital::cgmath::{Point3, Rad};
use orbital::ecs::{IntoSystem, Res, ResMut, System, World};
use orbital::ecs_bridge::{
    ActiveCamera, CameraDescriptorEcs, CursorGrabConfig, DeltaTime, EnvironmentDescriptorResource,
    Position, Rotation,
};
use orbital::logging::{self, error, info};
use orbital::resources::{
    GeneratedSkyParameters, SamplingType, SunPosition, WorldEnvironmentDescriptor,
};

pub const NAME: &str = "Orbital-Demo-Project: SkyBox";

pub fn entrypoint(
    event_loop_result: Result<
        orbital::winit::event_loop::EventLoop<()>,
        orbital::winit::error::EventLoopError,
    >,
) {
    #[cfg(not(target_os = "android"))]
    logging::init();

    let event_loop = event_loop_result.expect("Event Loop failure");

    let mut app_settings = AppSettings::default();
    app_settings.vsync_enabled = true;
    app_settings.name = NAME.to_string();
    app_settings.back_presses_to_exit = 3;

    match App::new()
        .add_module(SkyboxModule)
        .liftoff(event_loop, app_settings)
    {
        Ok(()) => info!("Cleanly exited!"),
        Err(e) => error!("Runtime failure: {e:?}"),
    }
}

orbital::make_main!(entrypoint);

struct SkyboxModule;

impl Module for SkyboxModule {
    fn setup(
        &self,
        ecs: &mut World,
        _device: &orbital::wgpu::Device,
        _queue: &orbital::wgpu::Queue,
    ) -> Vec<Box<dyn System>> {
        // Spawn camera entity
        let camera = ecs.spawn_entity();
        ecs.attach_component(
            &camera,
            CameraDescriptorEcs {
                label: "Default".into(),
                aspect: 16.0 / 9.0,
                fovy: Rad(std::f32::consts::FRAC_PI_4),
                near: 0.1,
                far: 10000.0,
                global_gamma: 2.2,
            },
        )
        .unwrap();
        ecs.attach_component(&camera, Position(Point3::new(0.0, 0.0, 3.0)))
            .unwrap();
        ecs.attach_component(&camera, Rotation::identity()).unwrap();
        ecs.insert_resource(ActiveCamera(camera));
        ecs.insert_resource(CursorGrabConfig(true));
        ecs.insert_resource(EnvironmentDescriptorResource(Some(
            WorldEnvironmentDescriptor::Generated {
                cube_face_size: DYNAMIC_SKY_CUBE_SIZE,
                sampling_type: SamplingType::GaussianBlur,
                custom_specular_mip_level_count: Some(DYNAMIC_SKY_MIP_LEVELS),
                parameters: Some(GeneratedSkyParameters {
                    sun_position: SunPosition::TimeOfDay { hours: 14.0 },
                    ..GeneratedSkyParameters::default()
                }),
                dynamic: true,
            },
        )));

        vec![
            sys_camera_controller.into_system(),
            sys_animate_sky(14.0).into_system(),
        ]
    }
}

const DYNAMIC_SKY_CUBE_SIZE: u32 = 256;
const DYNAMIC_SKY_MIP_LEVELS: u32 = 3;

fn sys_animate_sky(
    initial_hours: f32,
) -> impl FnMut(Res<DeltaTime>, ResMut<EnvironmentDescriptorResource>) {
    let mut clock = initial_hours;

    // The in-place dynamic sky path makes per-frame updates cheap, so there is
    // no throttling here — the sky updates every frame.
    move |dt: Res<DeltaTime>, mut descriptor: ResMut<EnvironmentDescriptorResource>| {
        let dt = dt.0 as f32;
        clock = (clock + dt / 240.0).rem_euclid(24.0);

        descriptor.0 = Some(WorldEnvironmentDescriptor::Generated {
            cube_face_size: DYNAMIC_SKY_CUBE_SIZE,
            sampling_type: SamplingType::GaussianBlur,
            custom_specular_mip_level_count: Some(DYNAMIC_SKY_MIP_LEVELS),
            parameters: Some(GeneratedSkyParameters {
                sun_position: SunPosition::TimeOfDay { hours: clock },
                ..GeneratedSkyParameters::default()
            }),
            dynamic: true,
        });
    }
}
