// Analytic skybox — evaluates the procedural `sky_color` per-pixel at full
// window resolution instead of sampling the regenerated specular cube (LoD 0).
//
// `sky_common.wgsl` (SkyParams + `sky_color`) is concatenated ahead of this
// file by `make_material_shader_descriptor(generated = true)`, so the skybox
// and the baked IBL reflections stay consistent. The cube texture is still
// regenerated for the reflection mips; only the background samples it no more.

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
    @location(0) clip_position: vec4<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;

// `SkyParams` comes from the prepended `sky_common.wgsl`.
@group(0) @binding(14) var<uniform> sky_params: SkyParams;

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
    // DIAG step 3: hardcode sky params to test if uniform buffer is the issue
    let hardcoded_params = SkyParams(
        vec3<f32>(0.5, 0.5, 0.0),  // sun_direction
        0.05, 0.0,  // sun_angular_radius, sun_intensity
        0.0, 0.0, 0.0,  // moon_angular_radius, moon_intensity, star_intensity
        0.0,  // star_density
        1.0,  // exposure (was 0.0)
        vec3<f32>(0.0),  // ground_albedo
        vec3<f32>(0.2, 0.4, 0.8),  // day_zenith (blue)
        vec3<f32>(0.0, 0.0, 0.0),
        vec3<f32>(0.0, 0.0, 0.0),
        vec3<f32>(0.0, 0.0, 0.0),
        vec3<f32>(0.0, 0.0, 0.0),
        vec3<f32>(0.0, 0.0, 0.0),
        vec3<f32>(0.0, 0.0, 0.0),
    );
    var world_environment_sample = sky_color(vec3<f32>(0.0, 1.0, 0.0), hardcoded_params);
    let aces_tone_mapped = aces_tone_map(world_environment_sample);
    return vec4<f32>(aces_tone_mapped, 1.0);
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
