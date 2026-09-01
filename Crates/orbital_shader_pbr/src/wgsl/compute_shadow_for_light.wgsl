fn compute_shadow_for_light(world_pos: vec3<f32>, view_depth: f32, light_idx: u32, normal: vec3<f32>) -> f32 {
    var factor = 1.0;
    let light = light_store[light_idx];

    var to_light: vec3<f32>;
    if (light.direction.w == LIGHT_TYPE_DIRECTIONAL) {
        to_light = normalize(-light.direction.xyz);
    } else {
        to_light = normalize(light.position.xyz - world_pos);
    }
    let n_dot_l = dot(normal, to_light);

    for (var i = 0u; i < shadow_data.cascade_count; i++) {
        let slot = shadow_data.slots[i];

        if (slot.light_index > light_idx) {
            break;
        }
        if (slot.light_index != light_idx) {
            continue;
        }

        let bias = slope_scaled_bias(slot.bias, n_dot_l);

        let world_bias = slot.bias * max(1.0 - clamp(n_dot_l, 0.0, 1.0), 0.01) * 2.0;
        let biased_world_pos = world_pos + to_light * world_bias;

        if (slot.shadow_type == SHADOW_TYPE_DIRECTIONAL_CASCADE) {
            if (view_depth > slot.cascade_split_depth) {
                continue;
            }
            let clip_pos = slot.light_view_proj * vec4<f32>(biased_world_pos, 1.0);
            let ndc = clip_pos.xyz / clip_pos.w;
            let shadow_coord = vec3<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5, ndc.z);
            if (all(shadow_coord.xy >= vec2<f32>(0.0)) && all(shadow_coord.xy < vec2<f32>(1.0)) && shadow_coord.z >= 0.0 && shadow_coord.z <= 1.0) {
                factor = sample_shadow_2d_pcf(slot.layer_index, shadow_coord, shadow_coord.z - bias);
                return factor;
            }
        } else if (slot.shadow_type == SHADOW_TYPE_SPOT) {
            let dist = length(light.position.xyz - world_pos);
            if (dist > slot.cascade_split_depth) {
                continue;
            }

            let bias_scale = clamp(SPOT_BIAS_SCALE / max(dist * dist, 0.1), 0.5, 5.0);

            let clip_pos = slot.light_view_proj * vec4<f32>(world_pos, 1.0);

            if (clip_pos.w <= 0.001) {
                return 1.0;
            }
            let ndc = clip_pos.xyz / clip_pos.w;
            let shadow_coord = vec3<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5, ndc.z);
            if (all(shadow_coord.xy >= vec2<f32>(0.0)) && all(shadow_coord.xy < vec2<f32>(1.0)) && shadow_coord.z >= 0.0 && shadow_coord.z <= 1.0) {
                factor = sample_shadow_2d_pcf(slot.layer_index, shadow_coord, shadow_coord.z - bias * bias_scale);
                return factor;
            }
            continue;
        } else if (slot.shadow_type == SHADOW_TYPE_POINT) {
            let light_pos = slot.light_view_proj[3].xyz;
            let frag_to_light = biased_world_pos - light_pos;
            let direction = normalize(frag_to_light);
            let abs_vec = abs(frag_to_light);
            let view_z = max(max(abs_vec.x, abs_vec.y), abs_vec.z);
            let far_plane = slot.cascade_split_depth;
            let near_plane = slot.near_plane;

            let depth = far_plane / (far_plane - near_plane) - far_plane * near_plane / (view_z * (far_plane - near_plane));

            factor = textureSampleCompare(
                point_shadow_maps, point_shadow_sampler,
                direction, i32(slot.layer_index), depth - bias
            );
            return factor;
        }
    }
    return 1.0;
}
