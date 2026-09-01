fn hdr_tone_map_gamma_correction(color: vec3<f32>) -> vec3<f32> {
    var result = color / (color + vec3<f32>(1.0));
    result = pow(result, vec3<f32>(1.0 / camera.global_gamma));
    return result;
}
