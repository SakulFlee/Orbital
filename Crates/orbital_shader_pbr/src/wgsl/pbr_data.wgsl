fn pbr_data(fragment_data: FragmentData) -> PBRData {
    let surface_data = surface(fragment_data);

    var out: PBRData;
    out.N = surface_data.normal;
    out.V = normalize(camera.position.xyz - fragment_data.world_position);
    let R = normalize(reflect(-out.V, out.N));
    out.NdotV = clamp(dot(out.N, out.V), 0.0, 1.0);
    out.albedo = surface_data.albedo;
    out.alpha = surface_data.alpha;
    out.metallic = surface_data.metallic;
    out.roughness = surface_data.roughness;
    out.occlusion = surface_data.occlusion;
    out.emissive = surface_data.emissive;

    out.ibl_diffuse = textureSample(
        diffuse_env_map,
        diffuse_sampler,
        out.N
    ).rgb;

    let specular_mip_count = textureNumLevels(specular_env_map);
    let specular_mip_level = out.roughness * out.roughness * f32(specular_mip_count - 1u);
    out.ibl_specular = textureSampleLevel(
        specular_env_map,
        specular_sampler,
        R,
        specular_mip_level
    ).rgb;

    out.brdf_lut = textureSample(
        ibl_brdf_lut_texture,
        ibl_brdf_lut_sampler,
        vec2<f32>(
            max(out.NdotV, 0.0001),
            clamp(out.roughness, 0.0001, 1.0)
        )).rg;

    return out;
}
