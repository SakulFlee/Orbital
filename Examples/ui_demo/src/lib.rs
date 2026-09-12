use orbital::ecs::{Commands, System, World};
use orbital::ecs_bridge::DeltaTime;
use orbital::logging::{self, error, info};
use orbital::app::{App, AppSettings, Module};
use orbital::ui::widgets;
use orbital::ui::layout::UiLayout;

pub fn entrypoint(
    event_loop_result: Result<
        orbital::winit::event_loop::EventLoop<()>,
        orbital::winit::error::EventLoopError,
    >,
) {
    #[cfg(not(target_os = "android"))]
    logging::init();

    let event_loop = event_loop_result.expect("Event Loop failure");

    let app_settings = AppSettings {
        vsync_enabled: true,
        name: "UI Demo".to_string(),
        back_presses_to_exit: 3,
        ..AppSettings::default()
    };

    match App::new()
        .add_module(UiDemoModule)
        .liftoff(event_loop, app_settings)
    {
        Ok(()) => info!("Cleanly exited!"),
        Err(e) => error!("Runtime failure: {e:?}"),
    }
}

orbital::make_main!(entrypoint);

struct UiDemoModule;

impl Module for UiDemoModule {
    fn setup(
        &self,
        ecs: &mut World,
        _device: &orbital::wgpu::Device,
        _queue: &orbital::wgpu::Queue,
    ) -> Vec<Box<dyn System>> {
        // Create a title text
        let title = widgets::create_text(
            ecs,
            "title",
            "UI Demo - Button, Text, TextBox, Checkbox",
            UiLayout::Absolute {
                x: 50.0,
                y: 30.0,
                width: 500.0,
                height: 40.0,
            },
        );

        // Create a button
        let play_button = widgets::create_button(
            ecs,
            "play_btn",
            "Play Game",
            UiLayout::Absolute {
                x: 100.0,
                y: 100.0,
                width: 200.0,
                height: 50.0,
            },
        );

        // Create a text box
        let username_input = widgets::create_textbox(
            ecs,
            "username",
            "Enter username...",
            UiLayout::Absolute {
                x: 100.0,
                y: 180.0,
                width: 250.0,
                height: 40.0,
            },
        );

        // Create a checkbox
        let terms_checkbox = widgets::create_checkbox(
            ecs,
            "terms",
            "I agree to the terms",
            UiLayout::Absolute {
                x: 100.0,
                y: 240.0,
                width: 250.0,
                height: 30.0,
            },
        );

        // Create another button
        let exit_button = widgets::create_button(
            ecs,
            "exit_btn",
            "Exit",
            UiLayout::Absolute {
                x: 100.0,
                y: 300.0,
                width: 200.0,
                height: 50.0,
            },
        );

        info!("UI Demo initialized with button, text box, and checkbox");

        vec![]
    }
}
