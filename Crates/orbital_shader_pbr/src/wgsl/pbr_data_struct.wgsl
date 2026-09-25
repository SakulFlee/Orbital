struct PBRData {
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
    occlusion: f32,
    emissive: vec3<f32>,
    ibl_diffuse: vec3<f32>,
    ibl_specular: vec3<f32>,
    brdf_lut: vec2<f32>,
    N: vec3<f32>,
    V: vec3<f32>,
    NdotV: f32,
}
