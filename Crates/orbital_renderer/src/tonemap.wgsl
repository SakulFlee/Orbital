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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel = 1.0 / vec2<f32>(textureDimensions(hdr_texture));
    let base = textureSample(hdr_texture, hdr_sampler, in.uv).rgb;
    let glare = sample_glare(in.uv, texel);
    let color = (base + glare) * params.exposure;

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
