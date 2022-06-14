use shared::entrypoint;

#[no_mangle]
pub extern "C" fn main_rs() {
    entrypoint();
}
