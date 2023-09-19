use wgpu::{CompositeAlphaMode, PresentMode};
use wgpu_engine::{RenderingEngine, WGPURenderingEngine};
use winit::{
    event_loop::EventLoopBuilder, platform::windows::EventLoopBuilderExtWindows,
    window::WindowBuilder,
};

#[test]
fn init_test() {
    // Make Window
    let event_loop = EventLoopBuilder::new().with_any_thread(true).build();
    let window = WindowBuilder::new()
        .build(&event_loop)
        .expect("[!NTF!] Window creation failed.");

    // Make Rendering Engine
    let rendering_engine = WGPURenderingEngine::new(&window);

    assert!(rendering_engine.is_ok());
}

#[test]
fn change_window_size() {
    // Make Window
    let event_loop = EventLoopBuilder::new().with_any_thread(true).build();
    let window = WindowBuilder::new()
        .build(&event_loop)
        .expect("[!NTF!] Window creation failed.");

    // Make Rendering Engine
    let mut rendering_engine =
        WGPURenderingEngine::new(&window).expect("[!NTF!] Engine creation failed");

    rendering_engine.change_window_size((123, 123));
    rendering_engine.reconfigure_surface();
}

#[test]
fn change_vsync() {
    // Make Window
    let event_loop = EventLoopBuilder::new().with_any_thread(true).build();
    let window = WindowBuilder::new()
        .build(&event_loop)
        .expect("[!NTF!] Window creation failed.");

    // Make Rendering Engine
    let mut rendering_engine =
        WGPURenderingEngine::new(&window).expect("[!NTF!] Engine creation failed");

    rendering_engine.change_vsync(PresentMode::AutoVsync);
    rendering_engine.reconfigure_surface();
}

#[test]
fn change_composite_alpha() {
    // Make Window
    let event_loop = EventLoopBuilder::new().with_any_thread(true).build();
    let window = WindowBuilder::new()
        .build(&event_loop)
        .expect("[!NTF!] Window creation failed.");

    // Make Rendering Engine
    let mut rendering_engine =
        WGPURenderingEngine::new(&window).expect("[!NTF!] Engine creation failed");

    rendering_engine.change_composite_alpha(CompositeAlphaMode::Auto);
    rendering_engine.reconfigure_surface();
}
