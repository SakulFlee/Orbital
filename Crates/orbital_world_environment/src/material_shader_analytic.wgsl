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
    // Precalculations
    let view_position = camera.perspective_projection_invert * in.clip_position;
    let view_ray_direction = view_position.xyz / view_position.w;
    var ray_direction = normalize((camera.view_projection_transposed * vec4(view_ray_direction, 0.0)).xyz);

    // A/B test: `sky_color` now takes individual fields (not the 176-byte
    // `SkyParams` struct by value), so we can call it directly again. If this
    // still renders black on the Adreno tablet, revert to the inline body.
    var world_environment_sample = sky_color(
        ray_direction,
        sky_params.sun_direction,
        sky_params.sun_angular_radius,
        sky_params.sun_intensity,
        sky_params.moon_angular_radius,
        sky_params.moon_intensity,
        sky_params.star_intensity,
        sky_params.star_density,
        sky_params.exposure,
        sky_params.ground_albedo,
        sky_params.day_zenith,
        sky_params.day_horizon,
        sky_params.night_zenith,
        sky_params.night_horizon,
        sky_params.twilight,
        sky_params.sun_color,
        sky_params.moon_color,
    );

    // ACES Tone Map (HDR mapping) — keeps the sun's gradient instead of
    // clamping it to a flat white core.
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
