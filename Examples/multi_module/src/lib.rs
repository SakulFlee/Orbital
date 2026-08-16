use orbital::app::{App, AppSettings};
use orbital::debug_render::DebugModule;
#[cfg(target_os = "android")]
use orbital::file_manager::FileManager;
use orbital::logging::{self, error, info};

mod modules;
use modules::camera_module::CameraModule;
use modules::model_module::ModelModule;
use winit::keyboard::KeyCode;

pub const NAME: &str = "Orbital-Demo-Project: MultiModule";

pub fn entrypoint(
    event_loop_result: Result<
        orbital::winit::event_loop::EventLoop<()>,
        orbital::winit::error::EventLoopError,
    >,
) {
    logging::init();

    let event_loop = event_loop_result.expect("Event Loop failure");

    #[cfg(target_os = "android")]
    {
        use orbital::winit::platform::android::EventLoopExtAndroid;
        let app = event_loop.android_app();
        FileManager::init_android_global(
            app.asset_manager(),
            app.internal_data_path(),
        )
        .expect("Failed to initialize FileManager for Android");
    }

    let mut app_settings = AppSettings::default();
    app_settings.vsync_enabled = true;
    app_settings.name = NAME.to_string();

    match App::new()
        .add_module(CameraModule)
        .add_module(ModelModule)
        // .add_module(LightModule)
        .add_module(
            DebugModule::new()
                .with_toggle_key(KeyCode::F3)
                .with_freeze_key(KeyCode::F4),
        )
        .liftoff(event_loop, app_settings)
    {
        Ok(()) => info!("Cleanly exited!"),
        Err(e) => error!("Runtime failure: {e:?}"),
    }
}

orbital::make_main!(entrypoint);
