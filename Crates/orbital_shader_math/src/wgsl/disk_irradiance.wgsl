fn disk_irradiance(
    disk_dir: vec3<f32>,
    radius: f32,
    radiance: vec3<f32>,
    N: vec3<f32>,
) -> vec3<f32> {
    let cos_n = max(dot(N, disk_dir), 0.0);
    if cos_n <= 0.0 {
        return vec3<f32>(0.0);
    }
    let solid_angle = TWO_PI * (1.0 - cos(radius));
    return radiance * solid_angle * cos_n;
}
