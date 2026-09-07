fn hammersley(i: u32, N: u32) -> vec2<f32> {
    let radical_inverse = radical_inverse(i);
    return vec2<f32>(f32(i) / f32(N), radical_inverse);
}
