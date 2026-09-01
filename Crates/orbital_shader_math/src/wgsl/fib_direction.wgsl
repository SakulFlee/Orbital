fn fib_direction(i: u32, n: u32) -> vec3<f32> {
    const GOLDEN_ANGLE: f32 = 2.399963229728653;
    let phi = f32(i) * GOLDEN_ANGLE;
    let y = 1.0 - (f32(i) + 0.5) / f32(n) * 2.0;
    let r = sqrt(max(0.0, 1.0 - y * y));
    return vec3<f32>(cos(phi) * r, y, sin(phi) * r);
}
