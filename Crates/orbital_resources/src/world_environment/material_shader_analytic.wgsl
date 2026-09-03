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

    // Inline sky computation for the skybox.
    //
    // NOTE: This is the body of the shared `sky_color(D, params)` function
    // from `sky_common.wgsl`, inlined here so we read the `sky_params`
    // uniform directly. Passing the 176-byte `SkyParams` struct by value into
    // a function is miscompiled on some Adreno Vulkan drivers (the standalone
    // call returns ~0 there), while the identical inline arithmetic is
    // correct — confirmed on a 4x4 diagnostic grid on the affected tablet.
    let D = ray_direction;
    let sun_dir = sky_params.sun_direction;
    let sun_elev = sun_dir.y; // in [-1, 1]

    let day_factor = smoothstep(-0.1, 0.25, sun_elev);
    let night_factor = 1.0 - day_factor;

    // --- Day / night sky gradient -----------------------------------------
    let zenith = mix(sky_params.night_zenith, sky_params.day_zenith, day_factor);
    let horizon = mix(sky_params.night_horizon, sky_params.day_horizon, day_factor);

    // Per-pixel vertical fade between horizon and zenith.
    let height = clamp(D.y, 0.0, 1.0);
    var colour = mix(horizon, zenith, pow(height, 0.3));

    // --- Twilight warm band -----------------------------------------------
    let twilight = exp(-abs(sun_elev) * 5.0);
    let twilight_visible = smoothstep(-0.15, 0.0, sun_elev);
    let horizon_term = pow(1.0 - height, 2.0);
    colour += sky_params.twilight * twilight * horizon_term * twilight_visible;

    // --- Sun disk + halo --------------------------------------------------
    let cos_a = clamp(dot(D, sun_dir), -1.0, 1.0);
    let sun_ang = acos(cos_a);
    let sun_visible = smoothstep(-0.05, 0.05, sun_elev);

    let core_sigma = sky_params.sun_angular_radius;
    let disk = exp(-0.5 * sun_ang * sun_ang / (core_sigma * core_sigma));
    colour += sky_params.sun_color * sky_params.sun_intensity * disk * sun_visible;

    let halo_sigma = sky_params.sun_angular_radius * 2.5;
    let halo = exp(-0.5 * sun_ang * sun_ang / (halo_sigma * halo_sigma));
    colour += sky_params.twilight * sky_params.sun_intensity * 0.1 * halo * sun_visible;

    // --- Moon (opposite the sun, visible at night) -------------------------
    let moon_dir = -sun_dir;
    let moon_ang = acos(clamp(dot(D, moon_dir), -1.0, 1.0));
    let moon_visible = 1.0 - smoothstep(-0.05, 0.05, sun_elev); // night only

    let moon_sigma = sky_params.moon_angular_radius;
    let moon_disk = exp(-pow(moon_ang / moon_sigma, 8.0));
    colour += sky_params.moon_color * sky_params.moon_intensity * moon_disk * moon_visible;

    let moon_halo_sigma = moon_sigma * 1.5;
    let moon_halo = exp(-moon_ang * moon_ang / (2.0 * moon_halo_sigma * moon_halo_sigma));
    colour += sky_params.moon_color * sky_params.moon_intensity * 0.1 * moon_halo * moon_visible;

    // --- Stars (deterministic, faded in at night) --------------------------
    const STAR_GRID: f32 = 90.0;
    let bias = i32(STAR_GRID);
    let cell = vec3<i32>(floor(D * STAR_GRID)) + vec3<i32>(bias);
    let h = star_hash(u32(cell.x), u32(cell.y), u32(cell.z));
    let bright = f32(h & 0xFFFFu) / 65535.0;

    let threshold = 1.0 - clamp(sky_params.star_density, 0.0, 1.0);
    if bright > threshold {
        let off = star_hash(
            u32(cell.x) + 0x9E3779B9u,
            u32(cell.y) + 0x85EBCA6Bu,
            u32(cell.z) + 0xC2B2AE35u,
        );
        let ox = f32(off & 0xFFu) / 255.0 - 0.5;
        let oy = f32((off >> 8u) & 0xFFu) / 255.0 - 0.5;
        let oz = f32((off >> 16u) & 0xFFu) / 255.0 - 0.5;
        let centre = (vec3<f32>(cell) - vec3<f32>(f32(bias))) + vec3<f32>(ox, oy, oz);
        let dist = length(D * STAR_GRID - centre);

        let star_val = exp(-dist * dist * 30.0);
        let star_bright = (bright - threshold) / (1.0 - threshold) * star_val;
        colour += vec3<f32>(0.85, 0.9, 1.0) * star_bright * sky_params.star_intensity
                * night_factor * 0.8;
    }

    // --- Ground (below the horizon) ----------------------------------------
    let ground_fade = smoothstep(-0.02, 0.02, D.y);
    let ground_tint = mix(0.03, 1.0, day_factor);
    colour = mix(sky_params.ground_albedo * ground_tint, colour, ground_fade);

    // --- Exposure ----------------------------------------------------------
    colour *= sky_params.exposure;

    // ACES Tone Map (HDR mapping) — keeps the sun's gradient instead of
    // clamping it to a flat white core.
    let aces_tone_mapped = aces_tone_map(colour);

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
