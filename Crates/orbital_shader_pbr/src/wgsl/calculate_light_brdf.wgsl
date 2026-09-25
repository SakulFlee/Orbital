fn calculate_light_brdf(light: Light, pbr: PBRData, world_position: vec3<f32>) -> LightContribution {
    var L: vec3<f32>;
    var light_distance: f32 = 1.0;
    var attenuation: f32 = 1.0;

    if (light.direction.w == LIGHT_TYPE_POINT) {
        L = light.position.xyz - world_position;
        light_distance = length(L);
        L = normalize(L);
        let atten_dist = max(light_distance, 0.5);
        attenuation = 1.0 / (atten_dist * atten_dist);
    } else if (light.direction.w == LIGHT_TYPE_DIRECTIONAL) {
        L = normalize(-light.direction.xyz);
        attenuation = 1.0;
    } else if (light.direction.w == LIGHT_TYPE_SPOT) {
        L = light.position.xyz - world_position;
        light_distance = length(L);
        L = normalize(L);
        let atten_dist = max(light_distance, 0.5);
        attenuation = 1.0 / (atten_dist * atten_dist);

        let cos_theta = dot(-L, normalize(light.direction.xyz));
        let angular = clamp(cos_theta * light.params.x + light.params.y, 0.0, 1.0);
        attenuation *= angular;
    } else {
        return LightContribution(vec3(0.0), 0.0);
    }

    let H = normalize(pbr.V + L);
    let NdotL = clamp(dot(pbr.N, L), 0.0, 1.0);
    let NdotH = clamp(dot(pbr.N, H), 0.0, 1.0);
    let VdotH = clamp(dot(pbr.V, H), 0.0, 1.0);

    var Lo: vec3<f32>;
    if NdotL > 0.0 {
        let D = distribution_ggx(NdotH, pbr.roughness);
        let G = schlick_smith_ggx(NdotL, pbr.NdotV, pbr.roughness);
        let F0 = mix(vec3(F0_DEFAULT), pbr.albedo, pbr.metallic);
        let F = fresnel_schlick(VdotH, F0);

        let nominator = D * F * G;
        let denominator = 4.0 * NdotL * pbr.NdotV + 0.0001;
        let specular = min(nominator / denominator, vec3(50.0));

        let kS = F;
        let kD = (vec3(1.0) - kS) * (1.0 - pbr.metallic);
        let diffuse = kD * pbr.albedo / PI;

        Lo += (diffuse + specular) * light.color.rgb * light.color.w * attenuation * NdotL;
    }
    return LightContribution(Lo, NdotL);
}
