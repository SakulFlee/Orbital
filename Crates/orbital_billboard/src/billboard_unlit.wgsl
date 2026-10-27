// Unlit billboard fragment entrypoint. The vertex entrypoint comes from the
// shared pbr_vertex node.

struct BillboardParams {
    color: vec4<f32>,
    params: vec4<f32>, // x = alpha cutoff
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var billboard_texture: texture_2d<f32>;
@group(1) @binding(1) var billboard_sampler: sampler;
@group(1) @binding(2) var<uniform> billboard_params: BillboardParams;

@fragment
fn entrypoint_fragment(in: FragmentData) -> @location(0) vec4<f32> {
    let sample = textureSample(billboard_texture, billboard_sampler, in.uv);
    let alpha = sample.a * billboard_params.color.a;
    if (alpha < billboard_params.params.x) {
        discard;
    }
    return vec4<f32>(sample.rgb * billboard_params.color.rgb, alpha);
}
