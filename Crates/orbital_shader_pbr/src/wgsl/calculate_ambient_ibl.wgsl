fn calculate_ambient_ibl(pbr: PBRData) -> vec3<f32> {
    let F0 = mix(vec3(F0_DEFAULT), pbr.albedo, pbr.metallic);
    let F = fresnel_schlick_roughness(pbr.NdotV, F0, pbr.roughness);

    let diffuse_color = (pbr.albedo * (vec3(1.0) - F) + 0.0001) * (1.0 - pbr.metallic + 0.0001);
    let diffuse_ibl = pbr.ibl_diffuse * diffuse_color;

    let specular_color = mix(F0, pbr.albedo, pbr.metallic);
    var specular_ibl = pbr.ibl_specular * (F * pbr.brdf_lut.x + pbr.brdf_lut.y);

    return (diffuse_ibl + specular_ibl) * pbr.occlusion * AMBIENT_INTENSITY;
}
