struct FxaaParams {
    inv_resolution: vec2<f32>,
    _padding: vec2<f32>,
};

@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(0) @binding(2) var<uniform> params: FxaaParams;

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

fn perceptual_luma(color: vec3<f32>) -> f32 {
    // The source is an sRGB target, so the sampled value is linear. FXAA's
    // edge threshold is defined in perceptual (gamma) space.
    let srgb = pow(max(color, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.2));
    return dot(srgb, vec3<f32>(0.299, 0.587, 0.114));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let texel = params.inv_resolution;

    let rgb_m = textureSample(source_texture, source_sampler, uv).rgb;
    let luma_m = perceptual_luma(rgb_m);

    let luma_nw = perceptual_luma(
        textureSample(source_texture, source_sampler, uv + vec2<f32>(-1.0, -1.0) * texel).rgb
    );
    let luma_ne = perceptual_luma(
        textureSample(source_texture, source_sampler, uv + vec2<f32>(1.0, -1.0) * texel).rgb
    );
    let luma_sw = perceptual_luma(
        textureSample(source_texture, source_sampler, uv + vec2<f32>(-1.0, 1.0) * texel).rgb
    );
    let luma_se = perceptual_luma(
        textureSample(source_texture, source_sampler, uv + vec2<f32>(1.0, 1.0) * texel).rgb
    );

    let luma_min = min(luma_m, min(min(luma_nw, luma_ne), min(luma_sw, luma_se)));
    let luma_max = max(luma_m, max(max(luma_nw, luma_ne), max(luma_sw, luma_se)));
    let range = luma_max - luma_min;

    if (range < max(0.0312, luma_max * 0.125)) {
        return vec4<f32>(rgb_m, 1.0);
    }

    var direction = vec2<f32>(
        -((luma_nw + luma_ne) - (luma_sw + luma_se)),
        (luma_nw + luma_sw) - (luma_ne + luma_se)
    );
    let direction_reduce = max(abs(direction), vec2<f32>(1.0 / 128.0));
    let direction_rcp_min = 1.0 / min(direction_reduce.x, direction_reduce.y);
    direction = clamp(direction * direction_rcp_min, vec2<f32>(-8.0), vec2<f32>(8.0)) * texel;

    let rgb_a = 0.5 * (
        textureSample(source_texture, source_sampler, uv + direction * (1.0 / 3.0 - 0.5)).rgb +
        textureSample(source_texture, source_sampler, uv + direction * (2.0 / 3.0 - 0.5)).rgb
    );
    let rgb_b = rgb_a * 0.5 + 0.25 * (
        textureSample(source_texture, source_sampler, uv + direction * -0.5).rgb +
        textureSample(source_texture, source_sampler, uv + direction * 0.5).rgb
    );

    let luma_b = perceptual_luma(rgb_b);
    if (luma_b < luma_min || luma_b > luma_max) {
        return vec4<f32>(rgb_a, 1.0);
    }
    return vec4<f32>(rgb_b, 1.0);
}
