// Froxel volumetric light injection.
//
// One invocation owns one froxel. It reconstructs the froxel center in world
// space, evaluates height fog plus every shadowed punctual/directional light,
// and stores (inscattering RGB, extinction A).

struct VolumetricParams {
    grid: vec4<u32>,     // x, y, z, unused
    media: vec4<f32>,    // density, scattering, extinction, anisotropy
    height: vec4<f32>,   // base height, falloff, density, max distance
    ambient: vec4<f32>,  // ambient fog color, IBL contribution multiplier
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(0) @binding(1) var<storage> light_store: array<Light>;
@group(0) @binding(2) var diffuse_env_map: texture_cube<f32>;
@group(0) @binding(3) var diffuse_env_sampler: sampler;
@group(0) @binding(8) var<uniform> shadow_data: ShadowData;
@group(0) @binding(9) var shadow_map_array: texture_depth_2d_array;
@group(0) @binding(10) var shadow_sampler: sampler_comparison;
@group(0) @binding(11) var point_shadow_maps: texture_depth_cube_array;
@group(0) @binding(12) var point_shadow_sampler: sampler_comparison;
@group(0) @binding(13) var<uniform> light_count: u32;

@group(1) @binding(0) var output_froxels: texture_storage_3d<rgba16float, write>;
@group(1) @binding(1) var<uniform> params: VolumetricParams;

fn volumetric_view_position(uv: vec2<f32>, ndc_z: f32) -> vec3<f32> {
    let position_h = camera.perspective_projection_invert
        * vec4<f32>(uv * 2.0 - 1.0, ndc_z, 1.0);
    return position_h.xyz / position_h.w;
}

@compute @workgroup_size(4, 4, 4)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let grid = params.grid.xyz;
    if (any(id >= grid)) {
        return;
    }

    let uv = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(grid.xy);
    // Froxel slices are distributed linearly in view-space distance. Using
    // NDC z directly packs almost every slice into the near field, which
    // makes room-distance fog and spot cones undersampled.
    let slice = (f32(id.z) + 0.5) / f32(grid.z);
    let view_depth = slice * params.height.w;

    let near_position = volumetric_view_position(uv, 0.0);
    let far_position = volumetric_view_position(uv, 1.0);
    let is_perspective = distance(near_position.xy, far_position.xy) > 1.0e-4;

    var view_position: vec3<f32>;
    if (is_perspective) {
        view_position = normalize(far_position) * view_depth;
    } else {
        view_position = vec3<f32>(near_position.xy, -view_depth);
    }

    let world_offset = (camera.view_projection_transposed * vec4<f32>(view_position, 0.0)).xyz;
    let world_position = camera.position.xyz + world_offset;

    if (view_depth > params.height.w) {
        textureStore(output_froxels, vec3<i32>(id), vec4<f32>(0.0));
        return;
    }

    var density = max(params.media.x, 0.0);
    density += params.height.z * exp(-params.height.y * max(world_position.y - params.height.x, 0.0));
    density = max(density, 0.0);
    if (density <= 1.0e-6) {
        textureStore(output_froxels, vec3<i32>(id), vec4<f32>(0.0));
        return;
    }

    let extinction = density * params.media.z;
    let view_direction = normalize(world_position - camera.position.xyz);
    let environment = textureSampleLevel(
        diffuse_env_map,
        diffuse_env_sampler,
        view_direction,
        0.0,
    ).rgb;
    var scattering = (params.ambient.rgb + environment * params.ambient.w)
        * density
        * params.media.y;
    let g = clamp(params.media.w, -0.95, 0.95);

    for (var i = 0u; i < light_count; i += 1u) {
        let light = light_store[i];
        if (light.color.w <= 0.0) {
            continue;
        }

        var to_light: vec3<f32>;
        var attenuation = 1.0;
        if (light.direction.w == LIGHT_TYPE_DIRECTIONAL) {
            to_light = normalize(-light.direction.xyz);
        } else {
            let delta = light.position.xyz - world_position;
            let distance_sq = dot(delta, delta);
            if (light.params.z > 0.0 && distance_sq > light.params.z) {
                continue;
            }
            let distance = sqrt(max(distance_sq, 0.0));
            to_light = delta / max(distance, 1.0e-4);
            attenuation = 1.0 / max(distance_sq, 0.25);

            if (light.direction.w == LIGHT_TYPE_SPOT) {
                let cos_theta = dot(-to_light, normalize(light.direction.xyz));
                attenuation *= clamp(cos_theta * light.params.x + light.params.y, 0.0, 1.0);
            }
        }

        let cos_theta = dot(view_direction, to_light);
        let phase_denom = max(1.0 + g * g - 2.0 * g * cos_theta, 1.0e-4);
        let phase = (1.0 - g * g) / (4.0 * 3.14159265 * pow(phase_denom, 1.5));

        let shadow = compute_shadow_for_light(
            world_position,
            view_depth,
            i,
            vec3<f32>(0.0, 1.0, 0.0),
        );

        scattering += light.color.rgb * light.color.w * attenuation * phase * shadow
            * density * params.media.y;
    }

    textureStore(
        output_froxels,
        vec3<i32>(id),
        vec4<f32>(max(scattering, vec3<f32>(0.0)), extinction),
    );
}
