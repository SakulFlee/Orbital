// Material surface: the data a material provides before lighting.
//
// Forward, G-buffer and shadow variants all consume this so custom materials
// do not need to reimplement lighting or target packing.

struct Surface {
    albedo: vec3<f32>,
    normal: vec3<f32>,
    roughness: f32,
    metallic: f32,
    emissive: vec3<f32>,
    occlusion: f32,
    alpha: f32,
}

fn surface(fragment_data: FragmentData) -> Surface {
    var out: Surface;
    out.normal = sample_normal_from_map(fragment_data);

    let albedo_sample = textureSample(
        albedo_texture,
        albedo_sampler,
        fragment_data.uv
    );
    let albedo_factored = albedo_sample.rgb * pbr_factors.albedo_factor.rgb;
    let albedo_clamped = clamp(albedo_factored, vec3(0.0), vec3(1.0));
    out.albedo = pow(albedo_clamped, vec3(camera.global_gamma));
    out.alpha = clamp(albedo_sample.a * pbr_factors.alpha_factor, 0.0, 1.0);

    let metallic_sample = textureSample(
        metallic_texture,
        metallic_sampler,
        fragment_data.uv
    ).r;
    out.metallic = clamp(metallic_sample * pbr_factors.metallic_factor, 0.0, 1.0);

    let roughness_sample = textureSample(
        roughness_texture,
        roughness_sampler,
        fragment_data.uv
    ).r;
    out.roughness = clamp(roughness_sample * pbr_factors.roughness_factor, 0.045, 0.9999);

    let occlusion_sample = textureSample(
        occlusion_texture,
        occlusion_sampler,
        fragment_data.uv
    ).r;
    out.occlusion = clamp(occlusion_sample, 0.0, 1.0);

    let emissive_sample = textureSample(
        emissive_texture,
        emissive_sampler,
        fragment_data.uv
    ).rgb;
    let emissive_clamped = clamp(emissive_sample, vec3(0.0), vec3(1.0));
    out.emissive = pow(emissive_clamped, vec3(camera.global_gamma));

    return out;
}
