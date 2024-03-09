use helper::log;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

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

    // TODO: Init winit

    // If needed, we can check the path (and query parameters) to route us:
    // match path.as_ref() {
    //     "/home" => Something here ...,
    //     _ => Ok(()),
    // }
    Ok(())
}
