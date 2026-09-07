// World environment texture-based skybox — entrypoint bodies.
//
// Expects the following to be defined by engine/math library nodes:
//   - CameraUniform (binding 0)
//   - Light, LIGHT_TYPE_* (binding 1)
//   - VertexOutput
//   - aces_tone_map

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(0) @binding(1) var<storage> light_store: array<Light>;
@group(0) @binding(2) var diffuse_env_map: texture_cube<f32>;
@group(0) @binding(3) var diffuse_env_sampler: sampler;
@group(0) @binding(4) var specular_env_map: texture_cube<f32>;
@group(0) @binding(5) var specular_env_sampler: sampler;

@vertex
fn entrypoint_vertex(
    @builtin(vertex_index) id: u32,
) -> VertexOutput {
    let uv = vec2<f32>(vec2<u32>(
        id & 1u,
        (id >> 1u) & 1u,
    ));

    var out: VertexOutput;
    out.clip_position = vec4(uv * 4.0 - 1.0, 1.0, 1.0);
    out.frag_position = out.clip_position;
    return out;
}

@fragment
fn entrypoint_fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let view_position = camera.perspective_projection_invert * in.clip_position;
    let view_ray_direction = view_position.xyz / view_position.w;
    var ray_direction = normalize((camera.view_projection_transposed * vec4(view_ray_direction, 0.0)).xyz);

    var world_environment_sample = textureSampleLevel(specular_env_map, specular_env_sampler, ray_direction, 0.0).rgb;

    let aces_tone_mapped = aces_tone_map(world_environment_sample);

    return vec4<f32>(aces_tone_mapped, 1.0);
}
