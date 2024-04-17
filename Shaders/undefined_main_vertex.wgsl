#import vertex_input.wgsl
#import vertex_output.wgsl

@vertex
fn main_vs(
    in: VertexInput
) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = vec4<f32>(in.position, 1.0);

    return out;
}
