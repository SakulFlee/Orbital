struct VertexOutput {
    @builtin(position) frag_position: vec4<f32>,
    // idk why this split is needed but these are actually two
    // different variables!! DO NOT REMOVE!!!
    @location(0) clip_position: vec4<f32>,
}

@vertex
fn entrypoint_vertex(
    @builtin(vertex_index) vertex_index: u32
) -> VertexOutput {
    let uv = vec2<f32>(vec2<u32>(
        vertex_index & 1u,
        (vertex_index >> 1u) & 1u,
    ));

    var out: VertexOutput;
    out.clip_position = vec4(uv * 4.0 - 1.0, 1.0, 1.0);
    out.frag_position = out.clip_position;
    return out;
}

@fragment
fn entrypoint_fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.clip_position;
}
