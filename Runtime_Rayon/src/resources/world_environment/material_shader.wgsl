// Light types
const LIGHT_TYPE_POINT: f32 = 0.0;
const LIGHT_TYPE_DIRECTIONAL: f32 = 1.0;
const LIGHT_TYPE_SPOT: f32 = 2.0;

struct CameraUniform {
    position: vec3<f32>,
    view_projection_matrix: mat4x4<f32>,
    perspective_view_projection_matrix: mat4x4<f32>,
    view_projection_transposed: mat4x4<f32>,
    perspective_projection_invert: mat4x4<f32>,
    global_gamma: f32,
}

struct Light {
    position: vec4<f32>,     // xyz: position, w: padding
    color: vec4<f32>,        // xyz: color, w: intensity
    direction: vec4<f32>,    // xyz: direction, w: type
    params: vec4<f32>,       // x: inner cone angle, y: outer cone angle, zw: padding
}

struct VertexOutput {
    @builtin(position) frag_position: vec4<f32>,
    // idk why this split is needed but these are actually two 
    // different variables!! DO NOT REMOVE!!!
    @location(0) clip_position: vec4<f32>,
}

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
    // Precalculations
    let view_position = camera.perspective_projection_invert * in.clip_position;
    let view_ray_direction = view_position.xyz / view_position.w;
    var ray_direction = normalize((camera.view_projection_transposed * vec4(view_ray_direction, 0.0)).xyz);

    // Sample HDRI WorldEnvironment as Sky Box, based on LoD (-1 = diffuse)
    var world_environment_sample = textureSampleLevel(specular_env_map, specular_env_sampler, ray_direction, 0.0).rgb;

    // Clamp sample to be within range (possible detail loss if there is data past >1.0)
    let clamped = clamp(world_environment_sample, vec3(0.0), vec3(1.0));

    // Adjust for gamma
    let gamma_adjustment = pow(clamped, vec3(camera.global_gamma));

    // ACES Tone Map (HDR mapping)
    let aces_tone_mapped = aces_tone_map(gamma_adjustment);

    return vec4<f32>(gamma_adjustment, 1.0);
}

// ACES tone mapping
const ACES_A: f32 = 2.51;
const ACES_B: f32 = 0.03;
const ACES_C: f32 = 2.43;
const ACES_D: f32 = 0.59;
const ACES_E: f32 = 0.14;
fn aces_tone_map(color: vec3<f32>) -> vec3<f32> {
    return clamp(
        (color * (ACES_A * color + ACES_B)) /
        (color * (ACES_C * color + ACES_D) + ACES_E),
        vec3(0.0),
        vec3(1.0)
    );
}
