struct VertexData {
    @location(0) position: vec4<f32>,
}

struct FragmentData {
    @builtin(position) position: vec4<f32>,
}

@vertex
fn entrypoint_vertex(
    vertex: VertexData
) -> FragmentData {
    var out: FragmentData;

    out.position = vertex.position;

    return out;
}
