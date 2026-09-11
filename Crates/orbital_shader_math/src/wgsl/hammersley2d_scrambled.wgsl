fn hammersley2d_scrambled(i: u32, N: u32, scramble: u32) -> vec2<f32> {
    return vec2(f32(i) / f32(N), radical_inverse_vdc(i ^ scramble));
}
