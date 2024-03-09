use helper::log;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{self, EventLoop},
    platform::web::WindowBuilderExtWebSys,
    window::WindowBuilder,
};

#[macro_use]
pub mod helper;

use crate::helper::{enable_panic_hook, query_html_element_first};

#[wasm_bindgen(start)]
pub async fn wasm_main() -> Result<(), JsValue> {
    log("Akimo-Project: Engine");
    log("(C) SakulFlee 2024");

    enable_panic_hook();

    let window = web_sys::window().unwrap();
    let location = window.location();

    let path = location.pathname().unwrap();
    let query = location.search().unwrap();

    #[cfg(debug_assertions)]
    console_log!("-> {}{}", path, query);

    let canvas: HtmlCanvasElement = query_html_element_first(&window, "#canvas")
        .await
        .expect("Failed to find #canvas")
        .dyn_into()
        .expect("Failed dynamically converting HtmlElement to HtmlCanvasElement!");
    console_log!("HtmlCanvasElement: {:?}", canvas);

    let event_loop = EventLoop::new().expect("EventLoop boot failed");
    let window = WindowBuilder::new()
        .with_canvas(Some(canvas))
        .build(&event_loop)
        .expect("Winit window/canvas creation failed");

    event_loop.set_control_flow(event_loop::ControlFlow::Poll);
    event_loop
        .run(move |event, window_target| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    println!("The close button was pressed; stopping");
                    window_target.exit();
                }
                Event::AboutToWait => {
                    // Application update code.

                    // Queue a RedrawRequested event.
                    //
                    // You only need to call this if you've determined that you need to redraw in
                    // applications which do not always need to. Applications that redraw continuously
                    // can render here instead.
                    window.request_redraw();
                }
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    // Redraw the application.
                    //
                    // It's preferable for applications that do not render continuously to render in
                    // this event rather than in AboutToWait, since rendering in here allows
                    // the program to gracefully handle redraws requested by the OS.

                    console_log!("REDRAW!!!");
                }
                _ => (),
            }
        })
        .expect("EventLoop failed to run!");

    // If needed, we can check the path (and query parameters) to route us:
    // match path.as_ref() {
    //     "/home" => Something here ...,
    //     _ => Ok(()),
    // }
    Ok(())
}
