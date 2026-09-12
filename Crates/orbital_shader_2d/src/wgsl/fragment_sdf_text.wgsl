// SDF (Signed Distance Field) text rendering shader
// Renders crisp text at any resolution using smoothstep

@group(1) @binding(0) var glyph_atlas: texture_2d<f32>;
@group(1) @binding(1) var glyph_sampler: sampler;

// SDF rendering parameters
struct SdfParams {
    smoothing: f32,      // Smoothing factor for anti-aliasing
    outline_width: f32,  // Width of outline (0 = no outline)
    outline_color: vec4<f32>,
};

@group(2) @binding(0) var<uniform> sdf_params: SdfParams;

fn fragment_sdf_text(input: Vertex2DOutput) -> @location(0) vec4<f32> {
    let sdf = textureSample(glyph_atlas, glyph_sampler, input.texcoord).r;

    // Smoothstep for anti-aliased edges
    let alpha = smoothstep(
        0.5 - sdf_params.smoothing,
        0.5 + sdf_params.smoothing,
        sdf
    );

    // Optional outline
    if (sdf_params.outline_width > 0.0) {
        let outline_alpha = smoothstep(
            0.5 - sdf_params.outline_width - sdf_params.smoothing,
            0.5 - sdf_params.outline_width + sdf_params.smoothing,
            sdf
        );
        let outline = mix(sdf_params.outline_color.rgb, input.color.rgb, alpha);
        return vec4<f32>(outline, mix(sdf_params.outline_color.a, input.color.a, alpha));
    }

    return vec4<f32>(input.color.rgb, input.color.a * alpha);
}
