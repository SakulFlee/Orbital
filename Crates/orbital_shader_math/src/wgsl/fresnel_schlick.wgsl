fn fresnel_schlick(cos_theta: f32, F0: vec3<f32>) -> vec3<f32> {
    let F = F0 + (1.0 - F0) * pow(1.0 - cos_theta, 5.0);
    return F;
}
