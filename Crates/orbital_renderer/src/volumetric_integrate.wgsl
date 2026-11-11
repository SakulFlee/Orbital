// Froxel column pre-integration.
//
// One invocation owns one froxel X/Y column. It walks the column from the
// camera to the maximum integration distance and stores cumulative
// in-scattering (RGB) and transmittance (A) at every step. The fullscreen
// composite then samples the prefix at the scene depth instead of integrating
// the volume per pixel.

struct IntegrateParams {
    grid: vec4<u32>,        // x, y, grid z, integration steps
    range: vec4<f32>,       // x = max integration distance
}

@group(0) @binding(0) var froxels: texture_3d<f32>;
@group(0) @binding(1) var output_prefix: texture_storage_3d<rgba16float, write>;
@group(0) @binding(2) var froxel_sampler: sampler;
@group(0) @binding(3) var<uniform> params: IntegrateParams;

@compute @workgroup_size(4, 4, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let grid = params.grid.xyz;
    if (any(id.xy >= grid.xy)) {
        return;
    }

    let steps = max(params.grid.w, 1u);
    let step_distance = params.range.x / f32(steps);
    let uv = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(grid.xy);

    var transmittance = 1.0;
    var scattering = vec3<f32>(0.0);

    for (var i = 0u; i < steps; i += 1u) {
        let t = (f32(i) + 0.5) / f32(steps);
        let sample = textureSampleLevel(
            froxels,
            froxel_sampler,
            vec3<f32>(uv, t),
            0.0,
        );
        scattering += transmittance * sample.rgb * step_distance;
        transmittance *= exp(-sample.a * step_distance);
        textureStore(
            output_prefix,
            vec3<i32>(vec2<i32>(id.xy), i32(i)),
            vec4<f32>(scattering, transmittance),
        );
    }
}
