#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[unsafe(no_mangle)]
pub extern "C" fn test(ptr: *mut u8, _len: usize) -> u64 {
    let pos_ptr = ptr as *mut Position;
    let pos = unsafe { &mut *pos_ptr };

    pos.x += 123.0;

    let out_ptr = ptr as u64;
    let out_len = std::mem::size_of::<Position>() as u64;

    (out_ptr << 32) | out_len
}
