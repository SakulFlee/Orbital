use wasm_bindgen::prelude::*;

/// Place for all JS/ES9 exported functions
#[wasm_bindgen]
extern "C" {
    /// JavaScripts `console.log`
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

// Create macro `console_log!` for easier usage
#[allow(unused_macros)]
#[macro_export]
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
	// `bare_bones`
	($($t:tt)*) => (
		console_log::log(&format_args!($($t)*).to_string())
	)
}
