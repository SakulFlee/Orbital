fn importance_sample_ggx(Xi: vec2<f32>, roughness: f32, N: vec3<f32>) -> vec3<f32> {
    let a = roughness * roughness;
    let phi = 2.0 * 3.14159 * Xi.x;
    let cos_theta = sqrt((1.0 - Xi.y) / (1.0 + (a*a - 1.0) * Xi.y));
    let sin_theta = sqrt(1.0 - cos_theta * cos_theta);
    let H = vec3(sin_theta * cos(phi), sin_theta * sin(phi), cos_theta);
    let up = select(select(vec3(0.0, 1.0, 0.0), vec3(1.0, 0.0, 0.0), abs(N.y) > 0.999), vec3(0.0, 1.0, 0.0), abs(N.z) > 0.999);
    let tangent = normalize(cross(up, N));
    let bitangent = cross(N, tangent);
    return normalize(tangent * H.x + bitangent * H.y + N * H.z);
}
