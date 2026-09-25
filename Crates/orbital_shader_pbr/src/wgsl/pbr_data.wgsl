fn pbr_data(fragment_data: FragmentData) -> PBRData {
    var out: PBRData;

    out.N = sample_normal_from_map(fragment_data);
    out.V = normalize(camera.position.xyz - fragment_data.world_position);
    let R = normalize(reflect(-out.V, out.N));
    out.NdotV = clamp(dot(out.N, out.V), 0.0, 1.0);

    let albedo_sample = textureSample(
        albedo_texture,
        albedo_sampler,
        fragment_data.uv
    ).rgb;
    let albedo_factored = albedo_sample * pbr_factors.albedo_factor.rgb;
    let albedo_clamped = clamp(albedo_factored, vec3(0.0), vec3(1.0));
    let albedo_gamma_applied = pow(albedo_clamped, vec3(camera.global_gamma));
    out.albedo = albedo_gamma_applied;

    let metallic_sample = textureSample(
        metallic_texture,
        metallic_sampler,
        fragment_data.uv
    ).r;
    let metallic_factored = metallic_sample * pbr_factors.metallic_factor;
    let metallic_clamped = clamp(metallic_factored, 0.0, 1.0);
    out.metallic = metallic_clamped;

    let roughness_sample = textureSample(
        roughness_texture,
        roughness_sampler,
        fragment_data.uv
    ).r;
    let roughness_factored = roughness_sample * pbr_factors.roughness_factor;
    let roughness_clamped = clamp(roughness_factored, 0.045, 0.9999);
    out.roughness = roughness_clamped;

    let occlusion_sample = textureSample(
        occlusion_texture,
        occlusion_sampler,
        fragment_data.uv
    ).r;
    let occlusion_clamped = clamp(occlusion_sample, 0.0, 1.0);
    out.occlusion = occlusion_clamped;

    let emissive_sample = textureSample(
        emissive_texture,
        emissive_sampler,
        fragment_data.uv
    ).rgb;
    let emissive_clamped = clamp(emissive_sample, vec3(0.0), vec3(1.0));
    let emissive_gamma_applied = pow(emissive_clamped, vec3(camera.global_gamma));
    out.emissive = emissive_gamma_applied;

    let diffuse_sample = textureSample(
        diffuse_env_map,
        diffuse_sampler,
        out.N
    ).rgb;
    out.ibl_diffuse = diffuse_sample;

    let specular_mip_count = textureNumLevels(specular_env_map);
    let specular_mip_level = out.roughness * out.roughness * f32(specular_mip_count - 1u);
    let specular_sample = textureSampleLevel(
        specular_env_map,
        specular_sampler,
        R,
        specular_mip_level
    ).rgb;
    out.ibl_specular = specular_sample;

    let brdf_lut_sample = textureSample(
        ibl_brdf_lut_texture,
        ibl_brdf_lut_sampler,
        vec2<f32>(
            max(out.NdotV, 0.0001),
            clamp(out.roughness, 0.0001, 1.0)
        )).rg;
    out.brdf_lut = brdf_lut_sample;

    return out;
}
