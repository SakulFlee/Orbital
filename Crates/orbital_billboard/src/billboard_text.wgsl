// SDF text billboard fragment entrypoint. The vertex entrypoint comes from the
// shared pbr_vertex node.

struct TextParams {
    color: vec4<f32>,
    outline_color: vec4<f32>,
    params: vec4<f32>, // x = outline width, y = smoothing, z = alpha cutoff
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var font_atlas: texture_2d<f32>;
@group(1) @binding(1) var font_sampler: sampler;
@group(1) @binding(2) var<uniform> text_params: TextParams;

@fragment
fn entrypoint_fragment(in: FragmentData) -> @location(0) vec4<f32> {
    let distance = textureSample(font_atlas, font_sampler, in.uv).r;
    let smoothing = max(text_params.params.y, 0.0001);
    let outline_width = max(text_params.params.x, 0.0);

    let inner = smoothstep(0.5 - smoothing, 0.5 + smoothing, distance);
    let outer = smoothstep(
        0.5 - smoothing - outline_width,
        0.5 + smoothing,
        distance
    );
    if (outer <= 0.0001) {
        discard;
    }

    let rgb = mix(text_params.outline_color.rgb, text_params.color.rgb, inner);
    var alpha = mix(text_params.outline_color.a, text_params.color.a, inner) * outer;
    if (alpha < text_params.params.z) {
        discard;
    }
    alpha = max(alpha, 0.0);
    return vec4<f32>(rgb, alpha);
}
