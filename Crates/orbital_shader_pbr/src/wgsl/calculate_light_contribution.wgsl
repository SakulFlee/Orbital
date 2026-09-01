fn calculate_light_contribution(pbr: PBRData, world_position: vec3<f32>, view_depth: f32) -> vec3<f32> {
    var Lo = vec3(0.0);
    for (var i = u32(0); i < light_count; i++) {
        let light = light_store[i];
        if (light.color.w == 0.0) { continue; }
        if (light.params.z > 0.0) {
            let dist_sq = dot(light.position.xyz - world_position, light.position.xyz - world_position);
            if (dist_sq > light.params.z) { continue; }
        }
        let contrib = calculate_light_brdf(light, pbr, world_position);
        if (contrib.ndotl <= 0.0) { continue; }
        let shadow = compute_shadow_for_light(world_position, view_depth, i, pbr.N);
        Lo += contrib.brdf * shadow;
    }
    return Lo;
}
