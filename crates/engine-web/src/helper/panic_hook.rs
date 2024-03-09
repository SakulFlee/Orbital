/// Checks if `debug_assertions` is enabled (i.e. build is in debug mode)
/// and if so, enables the panic-hook.
///
/// This panic-hook will log any thrown panic into the JavaScript console.
/// This is wanted for debug builds, but should be excluded from release / deployment versions.
/// Enabling / Using this also causes the resulting binary to get much bigger,
/// as it requires a lot of Rust-FMT functions.
/// Larger binaries are unwanted for WASM-Binaries.
pub fn enable_panic_hook() {
	#[cfg(debug_assertions)]
	hook();
}

fn hook() {
	console_log!("Enabling panic-hook! This is only intended for debugging.");
	console_error_panic_hook::set_once();
}
