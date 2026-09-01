fn importance_sample_lambert(uv: vec2<f32>) -> vec3<f32> {
    let phi = uv.y * TWO_PI;
    let cos_theta = sqrt(1.0 - uv.x);
    let sin_theta = sqrt(uv.x);
    return vec3(sin_theta * cos(phi), sin_theta * sin(phi), cos_theta);
}
