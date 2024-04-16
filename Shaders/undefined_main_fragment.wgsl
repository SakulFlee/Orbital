#import vertex_input.wgsl
#import vertex_output.wgsl

@fragment
fn main_fs(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
