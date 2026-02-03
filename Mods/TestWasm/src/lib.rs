use borsh::BorshDeserialize;
use orbital_variant::Variant;

#[unsafe(no_mangle)]
pub extern "C" fn test(ptr: *mut u8, len: usize) -> u64 {
    let input_slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    let input_variant = Variant::try_from_slice(input_slice).unwrap();

    let output_variant = match input_variant {
        Variant::F32(x) => Variant::F32(x + 123.0),
        _ => Variant::String("Invalid type!".to_string()),
    };
    let output_bytes = borsh::to_vec(&output_variant).unwrap();

    let output_ptr = output_bytes.as_ptr() as u64;
    let output_len = output_bytes.len() as u64;

    std::mem::forget(output_bytes);

    (output_ptr << 32) | output_len
}
