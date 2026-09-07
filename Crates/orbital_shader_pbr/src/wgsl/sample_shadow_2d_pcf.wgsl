fn sample_shadow_2d_pcf(layer: u32, shadow_coord: vec3<f32>, depth: f32) -> f32 {
    let dims = vec2<f32>(textureDimensions(shadow_map_array));
    let texel = 1.0 / dims;
    let clamped_uv = clamp(shadow_coord.xy, texel, vec2<f32>(1.0) - texel);
    var sum = 0.0;
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel;
            sum += textureSampleCompare(
                shadow_map_array, shadow_sampler,
                clamped_uv + offset, i32(layer), depth
            );
        }
    }
    return sum / 9.0;
}
