struct TonemapParams {
    exposure: f32,
    gamma: f32,
    bloom_threshold: f32,
    bloom_intensity: f32,
    _padding: vec4<f32>,
};

@group(0) @binding(0) var hdr_texture: texture_2d<f32>;
@group(0) @binding(1) var hdr_sampler: sampler;
@group(0) @binding(2) var<uniform> params: TonemapParams;
@group(0) @binding(3) var scene_depth: texture_depth_2d;

struct GlareLight {
    position: vec4<f32>,
    color: vec4<f32>,
};

struct GlareParams {
    view_projection: mat4x4<f32>,
    camera_position: vec4<f32>,
    // xy = viewport size, z = active light count, w = glare radius.
    viewport: vec4<f32>,
    lights: array<GlareLight, 32>,
};

@group(0) @binding(4) var<uniform> glare: GlareParams;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) id: u32) -> VertexOutput {
    let uv = vec2<f32>(
        f32((id << 1u) & 2u),
        f32(id & 2u),
    );

    var out: VertexOutput;
    out.position = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(uv.x, 1.0 - uv.y);
    return out;
}

/// A compact cross-shaped glare kernel sampled from the HDR scene before
/// tonemapping. The wider taps give light sources a visible halo without a
/// separate blur pass and keep the cost bounded for every AA mode.
fn sample_glare(uv: vec2<f32>, texel: vec2<f32>) -> vec3<f32> {
    if (params.bloom_intensity <= 0.0) {
        return vec3<f32>(0.0);
    }

    let offsets = array<vec2<f32>, 12>(
        vec2<f32>(-3.0, 0.0),
        vec2<f32>(3.0, 0.0),
        vec2<f32>(0.0, -3.0),
        vec2<f32>(0.0, 3.0),
        vec2<f32>(-2.0, -2.0),
        vec2<f32>(2.0, -2.0),
        vec2<f32>(-2.0, 2.0),
        vec2<f32>(2.0, 2.0),
        vec2<f32>(-7.0, 0.0),
        vec2<f32>(7.0, 0.0),
        vec2<f32>(0.0, -7.0),
        vec2<f32>(0.0, 7.0),
    );

    var glare = vec3<f32>(0.0);
    for (var i = 0u; i < 12u; i = i + 1u) {
        let sample_uv = clamp(uv + offsets[i] * texel, vec2<f32>(0.0), vec2<f32>(1.0));
        let sample = textureSample(hdr_texture, hdr_sampler, sample_uv).rgb;
        glare += max(sample - vec3<f32>(params.bloom_threshold), vec3<f32>(0.0));
    }

    return glare * (params.bloom_intensity / 12.0);
}

/// Screen-space glare around positional lights. The scene depth sample keeps
/// lights behind opaque geometry from leaking through walls; this does not
/// require a visible emissive mesh in the scene.
fn sample_light_glare(uv: vec2<f32>) -> vec3<f32> {
    let count = u32(glare.viewport.z);
    if (params.bloom_intensity <= 0.0 || count == 0u) {
        return vec3<f32>(0.0);
    }

    let viewport = glare.viewport.xy;
    let aspect = viewport.x / max(viewport.y, 1.0);
    let radius2 = glare.viewport.w * glare.viewport.w;
    var light_glare = vec3<f32>(0.0);

    for (var i = 0u; i < count; i = i + 1u) {
        let light = glare.lights[i];
        let clip = glare.view_projection * vec4<f32>(light.position.xyz, 1.0);
        if (clip.w <= 0.0) {
            continue;
        }

        let ndc = clip.xy / clip.w;
        if (abs(ndc.x) > 1.1 || abs(ndc.y) > 1.1) {
            continue;
        }

        let light_uv = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
        let pixel = vec2<i32>(
            clamp(
                light_uv * viewport,
                vec2<f32>(0.0),
                viewport - vec2<f32>(1.0),
            ),
        );
        let scene_depth_value = textureLoad(scene_depth, pixel, 0);
        let light_depth = clip.z / clip.w;
        if (light_depth > scene_depth_value + 0.0005) {
            continue;
        }

        let delta = (uv - light_uv) * vec2<f32>(aspect, 1.0);
        let distance2 = dot(delta, delta);
        let core = exp(-distance2 / (radius2 * 0.33));
        let halo = exp(-distance2 / (radius2 * 3.0)) * 0.35;
        let to_light = light.position.xyz - glare.camera_position.xyz;
        let attenuation = 1.0 / (1.0 + dot(to_light, to_light) * 0.015);
        let energy = light.color.w * 0.05 * attenuation * (core + halo);
        light_glare += light.color.rgb * energy;
    }

    return light_glare * params.bloom_intensity;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel = 1.0 / vec2<f32>(textureDimensions(hdr_texture));
    let base = textureSample(hdr_texture, hdr_sampler, in.uv).rgb;
    let scene_glare = sample_glare(in.uv, texel);
    let light_glare = sample_light_glare(in.uv);
    let color = (base + scene_glare + light_glare) * params.exposure;

    // ACES filmic curve.
    let mapped = clamp(
        (color * (2.51 * color + 0.03)) /
        (color * (2.43 * color + 0.59) + 0.14),
        vec3<f32>(0.0),
        vec3<f32>(1.0)
    );

    // Linear targets need the transfer function here; sRGB targets apply it
    // in hardware, so gamma is 1.0 in that case.
    let corrected = pow(mapped, vec3<f32>(params.gamma));
    return vec4<f32>(corrected, 1.0);
}
