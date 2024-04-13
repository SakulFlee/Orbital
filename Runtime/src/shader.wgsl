struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn main_vs(
    in: VertexInput
) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = vec4<f32>(in.position, 1.0);

    return out;
}

@fragment
fn main_fs(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
