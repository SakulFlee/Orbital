fn calculate_light_index_contribution(
    pbr: PBRData,
    world_position: vec3<f32>,
    view_depth: f32,
    light_index: u32,
) -> vec3<f32> {
    if (light_index >= light_count) {
        return vec3(0.0);
    }

    let light = light_store[light_index];
    if (light.color.w == 0.0) {
        return vec3(0.0);
    }
    if (light.params.z > 0.0) {
        let dist_sq = dot(light.position.xyz - world_position, light.position.xyz - world_position);
        if (dist_sq > light.params.z) {
            return vec3(0.0);
        }
    }

    let contrib = calculate_light_brdf(light, pbr, world_position);
    if (contrib.ndotl <= 0.0) {
        return vec3(0.0);
    }

    let shadow = compute_shadow_for_light(world_position, view_depth, light_index, pbr.N);
    return contrib.brdf * shadow;
}

fn calculate_light_contribution(pbr: PBRData, world_position: vec3<f32>, view_depth: f32) -> vec3<f32> {
    var Lo = vec3(0.0);
    for (var i = u32(0); i < light_count; i++) {
        Lo += calculate_light_index_contribution(pbr, world_position, view_depth, i);
    }
    return Lo;
}
