struct VertexOutput {
    @builtin(position) frag_position: vec4<f32>,
    @location(0) clip_position: vec4<f32>,
}
